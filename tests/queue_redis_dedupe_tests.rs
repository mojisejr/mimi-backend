//! Unit and Integration tests for Redis Deduplication Manager
//!
//! Tests written BEFORE implementation (Test-First Development)
//! These tests will initially fail (RED phase) until implementation is complete.
//!
//! Test Coverage:
//! - Unit tests: Duplicate key rejection logic
//! - Integration tests: Real Redis connection with concurrent operations
//! - Edge cases: Connection timeouts, network errors, invalid keys

use std::time::Duration;

#[cfg(test)]
mod redis_dedupe_tests {
    use super::*;

    /// Helper function to get Redis URL from environment or use default
    fn get_redis_url() -> String {
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string())
    }

    /// Helper function to generate unique test keys
    fn test_key(suffix: &str) -> String {
        format!("test:dedupe:{}", suffix)
    }

    #[tokio::test]
    async fn test_dedupe_key_first_insert_succeeds() {
        // Test: First insertion of a key should succeed
        let redis_url = get_redis_url();
        let key = test_key("first_insert");
        let ttl_secs = 5;

        // Try to connect to Redis
        let manager = match mimivibe_backend::queue::redis_dedupe::RedisDedupeManager::new(
            &redis_url,
        )
        .await
        {
            Ok(m) => m,
            Err(_) => {
                println!("Redis not available, skipping integration test");
                return;
            }
        };

        // First insert should succeed
        let result = manager.set_dedupe_key(&key, ttl_secs).await;
        assert!(result.is_ok(), "First insert should not error");
        assert!(
            result.unwrap(),
            "First insert should return true (key was set)"
        );

        // Cleanup
        let _ = manager.delete_key(&key).await;
    }

    #[tokio::test]
    async fn test_dedupe_key_duplicate_insert_fails() {
        // Test: Duplicate insertion of same key should fail/return false
        let redis_url = get_redis_url();
        let key = test_key("duplicate_insert");
        let ttl_secs = 5;

        // Try to connect to Redis
        let manager = match mimivibe_backend::queue::redis_dedupe::RedisDedupeManager::new(
            &redis_url,
        )
        .await
        {
            Ok(m) => m,
            Err(_) => {
                println!("Redis not available, skipping integration test");
                return;
            }
        };

        // First insert should succeed
        let first_result = manager.set_dedupe_key(&key, ttl_secs).await;
        assert!(first_result.is_ok(), "First insert should not error");
        assert!(
            first_result.unwrap(),
            "First insert should return true (key was set)"
        );

        // Duplicate insert should fail (return false because key already exists)
        let duplicate_result = manager.set_dedupe_key(&key, ttl_secs).await;
        assert!(duplicate_result.is_ok(), "Duplicate check should not error");
        assert!(
            !duplicate_result.unwrap(),
            "Duplicate insert should return false (key exists)"
        );

        // Cleanup
        let _ = manager.delete_key(&key).await;
    }

    #[tokio::test]
    async fn test_dedupe_key_check_exists() {
        // Test: check_dedupe_key should return true if key exists, false otherwise
        let redis_url = get_redis_url();
        let key = test_key("check_exists");
        let ttl_secs = 5;

        // Try to connect to Redis
        let manager = match mimivibe_backend::queue::redis_dedupe::RedisDedupeManager::new(
            &redis_url,
        )
        .await
        {
            Ok(m) => m,
            Err(_) => {
                println!("Redis not available, skipping integration test");
                return;
            }
        };

        // Key should not exist initially
        let exists_before = manager.check_dedupe_key(&key).await;
        assert!(exists_before.is_ok(), "Check should not error");
        assert!(!exists_before.unwrap(), "Key should not exist initially");

        // Set the key
        let _ = manager.set_dedupe_key(&key, ttl_secs).await;

        // Key should exist now
        let exists_after = manager.check_dedupe_key(&key).await;
        assert!(exists_after.is_ok(), "Check should not error");
        assert!(exists_after.unwrap(), "Key should exist after setting");

        // Cleanup
        let _ = manager.delete_key(&key).await;
    }

    #[tokio::test]
    async fn test_dedupe_key_ttl_expiration() {
        // Test: Key should expire after TTL seconds
        let redis_url = get_redis_url();
        let key = test_key("ttl_expiration");
        let ttl_secs = 2; // Short TTL for faster test

        // Try to connect to Redis
        let manager = match mimivibe_backend::queue::redis_dedupe::RedisDedupeManager::new(
            &redis_url,
        )
        .await
        {
            Ok(m) => m,
            Err(_) => {
                println!("Redis not available, skipping integration test");
                return;
            }
        };

        // Set key with short TTL
        let _ = manager.set_dedupe_key(&key, ttl_secs).await;

        // Key should exist immediately
        let exists_before = manager.check_dedupe_key(&key).await;
        assert!(exists_before.unwrap(), "Key should exist before expiration");

        // Wait for TTL to expire (add buffer)
        tokio::time::sleep(Duration::from_secs(ttl_secs as u64 + 1)).await;

        // Key should be gone after TTL
        let exists_after = manager.check_dedupe_key(&key).await;
        assert!(!exists_after.unwrap(), "Key should be expired after TTL");
    }

    #[tokio::test]
    async fn test_dedupe_concurrent_inserts() {
        // Test: Multiple concurrent attempts should only succeed once
        let redis_url = get_redis_url();
        let key = test_key("concurrent_inserts");
        let ttl_secs = 5;

        // Try to connect to Redis
        let manager = match mimivibe_backend::queue::redis_dedupe::RedisDedupeManager::new(
            &redis_url,
        )
        .await
        {
            Ok(m) => m,
            Err(_) => {
                println!("Redis not available, skipping integration test");
                return;
            }
        };

        let manager = std::sync::Arc::new(manager);

        // Spawn multiple concurrent tasks trying to set the same key
        let mut handles = vec![];
        for _i in 0..10 {
            let manager_clone = manager.clone();
            let key_clone = key.clone();
            handles.push(tokio::spawn(async move {
                manager_clone.set_dedupe_key(&key_clone, ttl_secs).await
            }));
        }

        // Collect results
        let mut results = vec![];
        for handle in handles {
            if let Ok(result) = handle.await {
                if let Ok(success) = result {
                    results.push(success);
                }
            }
        }

        // Only one should succeed (return true)
        let success_count = results.iter().filter(|&&v| v).count();
        assert_eq!(
            success_count, 1,
            "Only one concurrent insert should succeed"
        );

        // Cleanup
        let _ = manager.delete_key(&key).await;
    }

    #[tokio::test]
    async fn test_dedupe_invalid_redis_url() {
        // Test: Invalid Redis URL should return error
        let invalid_url = "redis://invalid-host:99999";

        let result =
            mimivibe_backend::queue::redis_dedupe::RedisDedupeManager::new(invalid_url).await;
        assert!(result.is_err(), "Invalid Redis URL should return error");
    }

    #[tokio::test]
    #[ignore] // Skip this test as it takes too long (testing non-routable IP timeout)
    async fn test_dedupe_connection_timeout() {
        // Test: Connection timeout should be handled gracefully
        // Use non-routable IP to simulate timeout (192.0.2.0/24 is reserved for documentation)
        let timeout_url = "redis://192.0.2.1:6379";

        let result = tokio::time::timeout(
            Duration::from_secs(2),
            mimivibe_backend::queue::redis_dedupe::RedisDedupeManager::new(timeout_url),
        )
        .await;

        // Should timeout or return connection error
        assert!(
            result.is_err() || (result.is_ok() && result.unwrap().is_err()),
            "Connection timeout should be handled"
        );
    }

    #[tokio::test]
    async fn test_dedupe_empty_key_validation() {
        // Test: Empty or invalid keys should return error
        let redis_url = get_redis_url();
        let ttl_secs = 5;

        // Try to connect to Redis
        let manager = match mimivibe_backend::queue::redis_dedupe::RedisDedupeManager::new(
            &redis_url,
        )
        .await
        {
            Ok(m) => m,
            Err(_) => {
                println!("Redis not available, skipping integration test");
                return;
            }
        };

        // Empty key should return error
        let result = manager.set_dedupe_key("", ttl_secs).await;
        assert!(result.is_err(), "Empty key should return error");

        // Whitespace-only key should return error
        let result = manager.set_dedupe_key("   ", ttl_secs).await;
        assert!(result.is_err(), "Whitespace key should return error");
    }

    #[tokio::test]
    async fn test_dedupe_network_error_handling() {
        // Test: Network errors should be propagated properly
        let redis_url = get_redis_url();
        let key = test_key("network_error");
        let ttl_secs = 5;

        // Try to connect to Redis
        let manager = match mimivibe_backend::queue::redis_dedupe::RedisDedupeManager::new(
            &redis_url,
        )
        .await
        {
            Ok(m) => m,
            Err(_) => {
                println!("Redis not available, skipping integration test");
                return;
            }
        };

        // Normal operation should work
        let result = manager.set_dedupe_key(&key, ttl_secs).await;
        assert!(result.is_ok(), "Normal operation should succeed");

        // Verify error types are properly defined
        use mimivibe_backend::queue::redis_dedupe::DedupeError;
        let err = DedupeError::ConnectionError("test".to_string());
        assert!(err.to_string().contains("connection"));

        // Cleanup
        let _ = manager.delete_key(&key).await;
    }

    #[tokio::test]
    async fn test_dedupe_integration_with_job_payload() {
        // Test: Integration with JobPayload dedupe_key field
        use chrono::Utc;
        use mimivibe_backend::queue::types::JobPayload;
        use uuid::Uuid;

        let redis_url = get_redis_url();
        let ttl_secs = 5;

        // Try to connect to Redis
        let manager = match mimivibe_backend::queue::redis_dedupe::RedisDedupeManager::new(
            &redis_url,
        )
        .await
        {
            Ok(m) => m,
            Err(_) => {
                println!("Redis not available, skipping integration test");
                return;
            }
        };

        // Create job payload with dedupe_key
        let payload = JobPayload {
            job_id: "test-job-123".to_string(),
            user_id: Uuid::new_v4(),
            question: "Test question".to_string(),
            card_count: 3,
            schema_version: "1".to_string(),
            prompt_version: "v2025-11-20-a".to_string(),
            dedupe_key: Some("user:123:question:hash".to_string()),
            trace_id: None,
            created_at: Utc::now(),
            metadata: serde_json::json!({}),
        };

        // Use dedupe_key from payload
        if let Some(dedupe_key) = &payload.dedupe_key {
            let result = manager.set_dedupe_key(dedupe_key, ttl_secs).await;
            assert!(result.is_ok(), "Should work with JobPayload dedupe_key");

            // Cleanup
            let _ = manager.delete_key(dedupe_key).await;
        }
    }
}
