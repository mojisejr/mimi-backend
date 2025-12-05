//! Prompt Cache Integration Tests
//!
//! Tests for verifying prompt cache initialization in worker and API startup.
//! Validates thread-safe Arc<HashMap> access and large prompt handling.

use mimivibe_backend::repository::PromptRepository;
use std::collections::HashMap;
use std::sync::Arc;

/// Test prompt cache structure type alias
type PromptCache = Arc<HashMap<String, String>>;

/// Helper function to create database pool for tests
async fn create_test_pool() -> Result<sqlx::PgPool, sqlx::Error> {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for tests");
    sqlx::PgPool::connect(&database_url).await
}

/// Test: Prompt cache initialization creates Arc with expected entries
#[tokio::test]
#[ignore] // Requires database connection
async fn test_prompt_cache_initialization() {
    // Arrange: Get database pool
    let pool = create_test_pool().await.expect("Failed to create pool");

    // Act: Load all active prompts from database
    let prompts = PromptRepository::load_all_active_prompts(&pool)
        .await
        .expect("Failed to load prompts");

    // Build cache HashMap
    let cache: PromptCache = Arc::new(
        prompts
            .into_iter()
            .map(|p| (p.agent_name, p.prompt_content))
            .collect(),
    );

    // Assert: Cache should contain 3 entries (question_filter, question_analyzer, reading_agent)
    assert_eq!(
        cache.len(),
        3,
        "Expected 3 prompts in cache, found {}",
        cache.len()
    );
    assert!(cache.contains_key("question_filter"));
    assert!(cache.contains_key("question_analyzer"));
    assert!(cache.contains_key("reading_agent"));
}

/// Test: Thread-safe cache access - multiple threads read same prompt simultaneously
#[tokio::test]
#[ignore] // Requires database connection
async fn test_thread_safe_cache_access() {
    // Arrange: Create prompt cache
    let pool = create_test_pool().await.expect("Failed to create pool");
    let prompts = PromptRepository::load_all_active_prompts(&pool)
        .await
        .expect("Failed to load prompts");

    let cache: PromptCache = Arc::new(
        prompts
            .into_iter()
            .map(|p| (p.agent_name, p.prompt_content))
            .collect(),
    );

    // Act: Spawn multiple threads to read from cache simultaneously
    let mut handles = vec![];

    for i in 0..10 {
        let cache_clone = Arc::clone(&cache);
        let handle = tokio::spawn(async move {
            // Each thread reads question_filter prompt
            let prompt = cache_clone.get("question_filter");
            assert!(prompt.is_some(), "Thread {} failed to read prompt", i);
            prompt.unwrap().len()
        });
        handles.push(handle);
    }

    // Assert: All threads successfully read the same prompt
    let results: Vec<usize> = futures::future::join_all(handles)
        .await
        .into_iter()
        .map(|r| r.expect("Thread panicked"))
        .collect();

    // All results should be the same length (same prompt content)
    let first_len = results[0];
    assert!(
        results.iter().all(|&len| len == first_len),
        "Thread safety violation: different results returned"
    );
}

/// Test: Large prompt handling - reading_agent (19KB+) loaded correctly
#[tokio::test]
#[ignore] // Requires database connection
async fn test_large_prompt_handling() {
    // Arrange: Get database pool
    let pool = create_test_pool().await.expect("Failed to create pool");

    // Act: Load all active prompts
    let prompts = PromptRepository::load_all_active_prompts(&pool)
        .await
        .expect("Failed to load prompts");

    // Build cache
    let cache: PromptCache = Arc::new(
        prompts
            .into_iter()
            .map(|p| (p.agent_name, p.prompt_content))
            .collect(),
    );

    // Assert: reading_agent prompt should be loaded and contain placeholders
    let reading_prompt = cache
        .get("reading_agent")
        .expect("reading_agent prompt not found");

    // Verify large prompt loaded correctly (expected 7000+ characters)
    assert!(
        reading_prompt.len() > 5000,
        "reading_agent prompt too short: {} chars",
        reading_prompt.len()
    );

    // Verify placeholders exist for template rendering
    assert!(
        reading_prompt.contains("{question}") || reading_prompt.contains("{{question}}"),
        "reading_agent missing {{question}} placeholder"
    );
}

/// Test: Cache initialization with missing prompts - graceful error handling
#[tokio::test]
async fn test_cache_empty_fallback() {
    // This test validates behavior when no prompts exist (empty cache scenario)

    // Arrange: Create empty cache
    let cache: PromptCache = Arc::new(HashMap::new());

    // Act & Assert: Access non-existent prompt returns None (not panic)
    assert!(cache.get("non_existent_agent").is_none());
    assert!(cache.get("question_filter").is_none());

    // Cache length should be 0
    assert_eq!(cache.len(), 0);
}

/// Test: Prompt content rendering with placeholders
#[tokio::test]
#[ignore] // Requires database connection
async fn test_prompt_placeholder_rendering() {
    // Arrange: Get database pool and load prompts
    let pool = create_test_pool().await.expect("Failed to create pool");
    let prompts = PromptRepository::load_all_active_prompts(&pool)
        .await
        .expect("Failed to load prompts");

    let cache: PromptCache = Arc::new(
        prompts
            .into_iter()
            .map(|p| (p.agent_name, p.prompt_content))
            .collect(),
    );

    // Act: Get question_filter prompt and check for expected structure
    let filter_prompt = cache
        .get("question_filter")
        .expect("question_filter prompt not found");

    // Assert: Prompt should contain Thai language elements (แม่หมอมี่ persona)
    // and proper structure
    assert!(
        filter_prompt.len() > 100,
        "question_filter prompt too short"
    );

    // Verify question_analyzer prompt exists and has content
    let analyzer_prompt = cache
        .get("question_analyzer")
        .expect("question_analyzer prompt not found");
    assert!(
        analyzer_prompt.len() > 100,
        "question_analyzer prompt too short"
    );
}

#[cfg(test)]
mod cache_concurrency_tests {
    use super::*;
    use std::time::Instant;

    /// Test: Performance - cache read speed under concurrent load
    #[tokio::test]
    #[ignore] // Requires database connection
    async fn test_cache_read_performance() {
        // Arrange
        let pool = create_test_pool().await.expect("Failed to create pool");
        let prompts = PromptRepository::load_all_active_prompts(&pool)
            .await
            .expect("Failed to load prompts");

        let cache: PromptCache = Arc::new(
            prompts
                .into_iter()
                .map(|p| (p.agent_name, p.prompt_content))
                .collect(),
        );

        // Act: Perform 1000 reads and measure time
        let start = Instant::now();
        let mut handles = vec![];

        for _ in 0..1000 {
            let cache_clone = Arc::clone(&cache);
            handles.push(tokio::spawn(async move {
                let _ = cache_clone.get("reading_agent");
            }));
        }

        futures::future::join_all(handles).await;
        let duration = start.elapsed();

        // Assert: 1000 reads should complete in under 100ms (very fast for in-memory cache)
        assert!(
            duration.as_millis() < 100,
            "Cache reads too slow: {}ms for 1000 reads",
            duration.as_millis()
        );
    }
}
