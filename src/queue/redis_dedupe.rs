//! Redis-based deduplication mechanism for job queue
//!
//! Provides atomic deduplication using Redis SETNX (SET if Not eXists) operations
//! with automatic TTL expiration. This prevents duplicate jobs from being enqueued
//! when the same dedupe_key is used within the TTL window.
//!
//! # Architecture
//!
//! - Uses Redis SETNX for atomic "check and set" operations
//! - Automatically expires keys after TTL to allow retries
//! - Thread-safe with Arc-wrapped connection manager
//! - Handles network errors and timeouts gracefully
//!
//! # Usage Example
//!
//! ```rust,no_run
//! use mimivibe_backend::queue::redis_dedupe::RedisDedupeManager;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let manager = RedisDedupeManager::new("redis://127.0.0.1:6379").await?;
//!
//! // Try to set a dedupe key with 60 second TTL
//! let was_set = manager.set_dedupe_key("job:user123:question", 60).await?;
//!
//! if was_set {
//!     println!("New job - proceed with enqueueing");
//! } else {
//!     println!("Duplicate job - skip enqueueing");
//! }
//! # Ok(())
//! # }
//! ```

use redis::{aio::ConnectionManager, AsyncCommands, Client, RedisError};
use std::sync::Arc;

/// Error types specific to Redis deduplication operations
#[derive(Debug)]
pub enum DedupeError {
    /// Redis connection error
    ConnectionError(String),
    /// Invalid key (empty or whitespace-only)
    InvalidKey(String),
    /// Redis operation error
    OperationError(String),
    /// Network timeout
    Timeout(String),
}

impl std::fmt::Display for DedupeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DedupeError::ConnectionError(msg) => write!(f, "Redis connection error: {}", msg),
            DedupeError::InvalidKey(msg) => write!(f, "Invalid dedupe key: {}", msg),
            DedupeError::OperationError(msg) => write!(f, "Redis operation error: {}", msg),
            DedupeError::Timeout(msg) => write!(f, "Redis timeout: {}", msg),
        }
    }
}

impl std::error::Error for DedupeError {}

impl From<RedisError> for DedupeError {
    fn from(err: RedisError) -> Self {
        if err.is_connection_dropped() || err.is_io_error() {
            DedupeError::ConnectionError(err.to_string())
        } else if err.is_timeout() {
            DedupeError::Timeout(err.to_string())
        } else {
            DedupeError::OperationError(err.to_string())
        }
    }
}

/// Redis-based deduplication manager
///
/// Manages job deduplication using Redis SETNX operations with TTL.
/// Uses connection manager for automatic reconnection and connection pooling.
#[derive(Clone)]
pub struct RedisDedupeManager {
    /// Redis connection manager (thread-safe, sharable)
    connection: Arc<ConnectionManager>,
}

impl RedisDedupeManager {
    /// Create a new RedisDedupeManager
    ///
    /// # Arguments
    ///
    /// * `redis_url` - Redis connection URL (e.g., "redis://127.0.0.1:6379")
    ///
    /// # Returns
    ///
    /// * `Ok(RedisDedupeManager)` - Successfully connected manager
    /// * `Err(DedupeError)` - Connection failed
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use mimivibe_backend::queue::redis_dedupe::RedisDedupeManager;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let manager = RedisDedupeManager::new("redis://127.0.0.1:6379").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new(redis_url: &str) -> Result<Self, DedupeError> {
        // Create Redis client
        let client = Client::open(redis_url).map_err(|e| {
            DedupeError::ConnectionError(format!("Failed to create Redis client: {}", e))
        })?;

        // Create connection manager (handles reconnection automatically)
        let connection = ConnectionManager::new(client).await.map_err(|e| {
            DedupeError::ConnectionError(format!("Failed to connect to Redis: {}", e))
        })?;

        Ok(Self {
            connection: Arc::new(connection),
        })
    }

    /// Check if a dedupe key exists in Redis
    ///
    /// # Arguments
    ///
    /// * `key` - The deduplication key to check
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - Key exists (duplicate detected)
    /// * `Ok(false)` - Key does not exist (no duplicate)
    /// * `Err(DedupeError)` - Redis operation failed
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use mimivibe_backend::queue::redis_dedupe::RedisDedupeManager;
    /// # async fn example(manager: &RedisDedupeManager) -> Result<(), Box<dyn std::error::Error>> {
    /// let exists = manager.check_dedupe_key("job:user123:question").await?;
    /// if exists {
    ///     println!("Job already queued or processing");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn check_dedupe_key(&self, key: &str) -> Result<bool, DedupeError> {
        // Validate key
        if key.trim().is_empty() {
            return Err(DedupeError::InvalidKey(
                "Dedupe key cannot be empty or whitespace".to_string(),
            ));
        }

        // Clone connection manager for async use
        let mut conn = self.connection.as_ref().clone();

        // Check if key exists using EXISTS command
        let exists: bool = conn.exists(key).await?;

        Ok(exists)
    }

    /// Atomically set a dedupe key with TTL (SETNX operation)
    ///
    /// Uses Redis SETNX (SET if Not eXists) for atomic check-and-set.
    /// If the key already exists, returns false without modifying it.
    /// If the key is set successfully, it will expire after `ttl_secs` seconds.
    ///
    /// # Arguments
    ///
    /// * `key` - The deduplication key to set
    /// * `ttl_secs` - Time-to-live in seconds (key will auto-expire)
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - Key was set successfully (no duplicate)
    /// * `Ok(false)` - Key already exists (duplicate detected)
    /// * `Err(DedupeError)` - Redis operation failed
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use mimivibe_backend::queue::redis_dedupe::RedisDedupeManager;
    /// # async fn example(manager: &RedisDedupeManager) -> Result<(), Box<dyn std::error::Error>> {
    /// let was_set = manager.set_dedupe_key("job:user123:question", 60).await?;
    /// if was_set {
    ///     println!("Enqueue the job");
    /// } else {
    ///     println!("Job is duplicate, skip enqueuing");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn set_dedupe_key(&self, key: &str, ttl_secs: u32) -> Result<bool, DedupeError> {
        // Validate key
        if key.trim().is_empty() {
            return Err(DedupeError::InvalidKey(
                "Dedupe key cannot be empty or whitespace".to_string(),
            ));
        }

        // Clone connection manager for async use
        let mut conn = self.connection.as_ref().clone();

        // Use SET with NX (only set if not exists) and EX (expiration) options
        // Redis command: SET key value NX EX ttl_secs
        // Returns: OK if set, nil if key already exists
        let result: Option<String> = redis::cmd("SET")
            .arg(key)
            .arg("1") // Value doesn't matter for dedupe, we just need the key
            .arg("NX") // Only set if key does Not eXist
            .arg("EX") // Set expiration
            .arg(ttl_secs)
            .query_async(&mut conn)
            .await?;

        // If result is Some("OK"), key was set successfully
        // If result is None, key already existed
        Ok(result.is_some())
    }

    /// Delete a dedupe key (for cleanup/testing)
    ///
    /// # Arguments
    ///
    /// * `key` - The deduplication key to delete
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Key was deleted or didn't exist
    /// * `Err(DedupeError)` - Redis operation failed
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use mimivibe_backend::queue::redis_dedupe::RedisDedupeManager;
    /// # async fn example(manager: &RedisDedupeManager) -> Result<(), Box<dyn std::error::Error>> {
    /// manager.delete_key("job:user123:question").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn delete_key(&self, key: &str) -> Result<(), DedupeError> {
        // Validate key
        if key.trim().is_empty() {
            return Err(DedupeError::InvalidKey(
                "Dedupe key cannot be empty or whitespace".to_string(),
            ));
        }

        // Clone connection manager for async use
        let mut conn = self.connection.as_ref().clone();

        // Delete the key (DEL command)
        let _: () = conn.del(key).await?;

        Ok(())
    }

    /// Get TTL (time-to-live) for a key in seconds
    ///
    /// # Arguments
    ///
    /// * `key` - The deduplication key to check
    ///
    /// # Returns
    ///
    /// * `Ok(Some(ttl))` - Key exists with TTL in seconds
    /// * `Ok(None)` - Key does not exist or has no TTL
    /// * `Err(DedupeError)` - Redis operation failed
    pub async fn get_ttl(&self, key: &str) -> Result<Option<i64>, DedupeError> {
        // Validate key
        if key.trim().is_empty() {
            return Err(DedupeError::InvalidKey(
                "Dedupe key cannot be empty or whitespace".to_string(),
            ));
        }

        // Clone connection manager for async use
        let mut conn = self.connection.as_ref().clone();

        // Get TTL in seconds
        let ttl: i64 = conn.ttl(key).await?;

        // Redis returns:
        // -2 if key does not exist
        // -1 if key exists but has no expiration
        // positive number for TTL in seconds
        match ttl {
            -2 => Ok(None), // Key doesn't exist
            -1 => Ok(None), // Key exists but no TTL (shouldn't happen with our usage)
            n if n > 0 => Ok(Some(n)),
            _ => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to get Redis URL from environment or use default
    fn get_redis_url() -> String {
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string())
    }

    #[tokio::test]
    async fn test_redis_dedupe_manager_creation() {
        let redis_url = get_redis_url();
        let result = RedisDedupeManager::new(&redis_url).await;

        // This might fail if Redis is not running, which is okay for unit tests
        if result.is_ok() {
            println!("Successfully connected to Redis");
        } else {
            println!("Redis not available: {:?}", result.err());
        }
    }

    #[tokio::test]
    async fn test_invalid_key_validation() {
        let redis_url = get_redis_url();
        if let Ok(manager) = RedisDedupeManager::new(&redis_url).await {
            // Empty key should fail
            let result = manager.check_dedupe_key("").await;
            assert!(result.is_err(), "Empty key should return error");

            // Whitespace key should fail
            let result = manager.check_dedupe_key("   ").await;
            assert!(result.is_err(), "Whitespace key should return error");
        }
    }

    #[tokio::test]
    async fn test_error_type_construction() {
        let err = DedupeError::InvalidKey("test".to_string());
        assert!(err.to_string().contains("Invalid dedupe key"));

        let err = DedupeError::ConnectionError("test".to_string());
        assert!(err.to_string().contains("connection error"));

        let err = DedupeError::OperationError("test".to_string());
        assert!(err.to_string().contains("operation error"));

        let err = DedupeError::Timeout("test".to_string());
        assert!(err.to_string().contains("timeout"));
    }
}
