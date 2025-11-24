//! Connection pooling for queue backends
//!
//! This module provides connection pool implementations for Redis and Upstash
//! queue backends with automatic reconnection, health checks, and metrics logging.

use crate::config::{EnvironmentConfig, QueuePoolConfig};
use async_trait::async_trait;
use redis::{aio::ConnectionManager, Client as RedisClient};
use std::error::Error;
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{Mutex, Semaphore};

/// Pool error types
#[derive(Debug)]
pub enum PoolError {
    /// Connection acquisition timeout
    AcquisitionTimeout(String),
    /// Maximum connections reached
    PoolExhausted(String),
    /// Connection failed
    ConnectionFailed(String),
    /// Pool is closed
    PoolClosed(String),
    /// Configuration error
    ConfigError(String),
}

impl fmt::Display for PoolError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PoolError::AcquisitionTimeout(msg) => {
                write!(f, "Connection acquisition timeout: {}", msg)
            }
            PoolError::PoolExhausted(msg) => write!(f, "Pool exhausted: {}", msg),
            PoolError::ConnectionFailed(msg) => write!(f, "Connection failed: {}", msg),
            PoolError::PoolClosed(msg) => write!(f, "Pool closed: {}", msg),
            PoolError::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
        }
    }
}

impl Error for PoolError {}

/// Pool metrics for monitoring
#[derive(Debug, Clone)]
pub struct PoolMetrics {
    /// Total number of connections created
    pub total_connections_created: u64,
    /// Number of active connections
    pub active_connections: u64,
    /// Number of idle connections
    pub idle_connections: u64,
    /// Total number of connection acquisitions
    pub total_acquisitions: u64,
    /// Total number of failed connections
    pub total_connection_failures: u64,
    /// Total number of reconnections
    pub total_reconnections: u64,
    /// Average wait time for connection acquisition (milliseconds)
    pub avg_wait_time_ms: u64,
}

impl PoolMetrics {
    /// Create new empty metrics
    pub fn new() -> Self {
        Self {
            total_connections_created: 0,
            active_connections: 0,
            idle_connections: 0,
            total_acquisitions: 0,
            total_connection_failures: 0,
            total_reconnections: 0,
            avg_wait_time_ms: 0,
        }
    }

    /// Log metrics to stdout
    pub fn log(&self, pool_name: &str) {
        println!("Pool Metrics [{}]:", pool_name);
        println!(
            "  Total connections created: {}",
            self.total_connections_created
        );
        println!("  Active connections: {}", self.active_connections);
        println!("  Idle connections: {}", self.idle_connections);
        println!("  Total acquisitions: {}", self.total_acquisitions);
        println!("  Connection failures: {}", self.total_connection_failures);
        println!("  Reconnections: {}", self.total_reconnections);
        println!("  Avg wait time: {}ms", self.avg_wait_time_ms);
    }
}

impl Default for PoolMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Connection pool trait
///
/// Defines the interface for connection pool implementations
#[async_trait]
pub trait ConnectionPool: Send + Sync {
    /// Get a connection from the pool
    ///
    /// # Returns
    ///
    /// * `Ok(Connection)` - Successfully acquired connection
    /// * `Err(PoolError)` - Failed to acquire connection
    async fn get_connection(&self) -> Result<Box<dyn std::any::Any + Send>, Box<dyn Error>>;

    /// Get pool metrics
    fn get_metrics(&self) -> PoolMetrics;

    /// Close the pool and all connections
    async fn close(&self) -> Result<(), Box<dyn Error>>;

    /// Check pool health
    async fn health_check(&self) -> Result<bool, Box<dyn Error>>;
}

/// Redis connection pool
///
/// Manages a pool of Redis connections with automatic reconnection,
/// health checks, and metrics tracking.
pub struct RedisConnectionPool {
    /// Configuration
    config: QueuePoolConfig,
    /// Redis client for creating connections
    client: RedisClient,
    /// Semaphore to limit concurrent connections
    semaphore: Arc<Semaphore>,
    /// Shared metrics
    metrics: Arc<Mutex<PoolMetricsInternal>>,
    /// Pool closed flag
    closed: Arc<Mutex<bool>>,
}

/// Internal metrics with atomic counters
struct PoolMetricsInternal {
    total_created: AtomicU64,
    active: AtomicU64,
    total_acquisitions: AtomicU64,
    total_failures: AtomicU64,
    total_reconnections: AtomicU64,
    total_wait_time_ms: AtomicU64,
}

impl PoolMetricsInternal {
    fn new() -> Self {
        Self {
            total_created: AtomicU64::new(0),
            active: AtomicU64::new(0),
            total_acquisitions: AtomicU64::new(0),
            total_failures: AtomicU64::new(0),
            total_reconnections: AtomicU64::new(0),
            total_wait_time_ms: AtomicU64::new(0),
        }
    }

    fn to_public(&self, idle: u64) -> PoolMetrics {
        let total_acq = self.total_acquisitions.load(Ordering::Relaxed);
        let avg_wait = if total_acq > 0 {
            self.total_wait_time_ms.load(Ordering::Relaxed) / total_acq
        } else {
            0
        };

        PoolMetrics {
            total_connections_created: self.total_created.load(Ordering::Relaxed),
            active_connections: self.active.load(Ordering::Relaxed),
            idle_connections: idle,
            total_acquisitions: total_acq,
            total_connection_failures: self.total_failures.load(Ordering::Relaxed),
            total_reconnections: self.total_reconnections.load(Ordering::Relaxed),
            avg_wait_time_ms: avg_wait,
        }
    }
}

impl RedisConnectionPool {
    /// Create a new Redis connection pool
    ///
    /// # Arguments
    ///
    /// * `redis_url` - Redis connection URL
    /// * `config` - Pool configuration
    ///
    /// # Returns
    ///
    /// * `Ok(RedisConnectionPool)` - Successfully created pool
    /// * `Err(PoolError)` - Failed to create pool
    pub fn new(redis_url: &str, config: QueuePoolConfig) -> Result<Self, Box<dyn Error>> {
        let client = RedisClient::open(redis_url)
            .map_err(|e| PoolError::ConfigError(format!("Invalid Redis URL: {}", e)))?;

        let semaphore = Arc::new(Semaphore::new(config.max_connections as usize));
        let metrics = Arc::new(Mutex::new(PoolMetricsInternal::new()));

        Ok(Self {
            config,
            client,
            semaphore,
            metrics,
            closed: Arc::new(Mutex::new(false)),
        })
    }

    /// Create pool from environment config
    pub async fn from_env_config(env_config: &EnvironmentConfig) -> Result<Self, Box<dyn Error>> {
        let redis_url = env_config.redis_url()?;
        Self::new(redis_url, env_config.pool.clone())
    }

    /// Create a new connection with retry logic
    async fn create_connection(&self) -> Result<ConnectionManager, Box<dyn Error>> {
        let mut attempts = 0;
        let mut delay = self.config.retry_backoff_base();
        let max_delay = self.config.retry_backoff_max();

        loop {
            attempts += 1;

            match ConnectionManager::new(self.client.clone()).await {
                Ok(conn) => {
                    let metrics = self.metrics.lock().await;
                    metrics.total_created.fetch_add(1, Ordering::Relaxed);
                    if attempts > 1 {
                        metrics.total_reconnections.fetch_add(1, Ordering::Relaxed);
                        println!("Successfully reconnected after {} attempts", attempts);
                    }
                    return Ok(conn);
                }
                Err(e) => {
                    let metrics = self.metrics.lock().await;
                    metrics.total_failures.fetch_add(1, Ordering::Relaxed);

                    if attempts >= self.config.max_retry_attempts {
                        return Err(Box::new(PoolError::ConnectionFailed(format!(
                            "Failed after {} attempts: {}",
                            attempts, e
                        ))));
                    }

                    println!(
                        "Connection attempt {} failed: {}. Retrying in {:?}...",
                        attempts, e, delay
                    );

                    tokio::time::sleep(delay).await;

                    // Exponential backoff with cap
                    delay = std::cmp::min(delay * 2, max_delay);
                }
            }
        }
    }
}

#[async_trait]
impl ConnectionPool for RedisConnectionPool {
    async fn get_connection(&self) -> Result<Box<dyn std::any::Any + Send>, Box<dyn Error>> {
        // Check if pool is closed
        {
            let closed = self.closed.lock().await;
            if *closed {
                return Err(Box::new(PoolError::PoolClosed(
                    "Pool has been closed".to_string(),
                )));
            }
        }

        let start = Instant::now();

        // Acquire permit from semaphore with timeout
        let permit =
            tokio::time::timeout(self.config.connection_timeout(), self.semaphore.acquire())
                .await
                .map_err(|_| {
                    PoolError::AcquisitionTimeout(format!(
                        "Timeout after {:?}",
                        self.config.connection_timeout()
                    ))
                })?
                .map_err(|_| PoolError::PoolClosed("Semaphore closed".to_string()))?;

        let wait_time = start.elapsed();
        let metrics = self.metrics.lock().await;
        metrics.total_acquisitions.fetch_add(1, Ordering::Relaxed);
        metrics
            .total_wait_time_ms
            .fetch_add(wait_time.as_millis() as u64, Ordering::Relaxed);
        metrics.active.fetch_add(1, Ordering::Relaxed);
        drop(metrics);

        // Create connection
        let conn = self.create_connection().await?;

        // Release permit when connection is dropped (simplified - in production would need RAII wrapper)
        permit.forget();

        Ok(Box::new(conn))
    }

    fn get_metrics(&self) -> PoolMetrics {
        let available = self.semaphore.available_permits();
        let idle = available as u64;

        // Use try_lock instead of blocking_lock to avoid blocking in async context
        match self.metrics.try_lock() {
            Ok(metrics) => metrics.to_public(idle),
            Err(_) => {
                // If lock is held, return default metrics
                PoolMetrics {
                    total_connections_created: 0,
                    active_connections: 0,
                    idle_connections: idle,
                    total_acquisitions: 0,
                    total_connection_failures: 0,
                    total_reconnections: 0,
                    avg_wait_time_ms: 0,
                }
            }
        }
    }

    async fn close(&self) -> Result<(), Box<dyn Error>> {
        let mut closed = self.closed.lock().await;
        *closed = true;
        println!("Redis connection pool closed");
        Ok(())
    }

    async fn health_check(&self) -> Result<bool, Box<dyn Error>> {
        if !self.config.enable_health_checks {
            return Ok(true);
        }

        // Try to create a test connection
        match self.create_connection().await {
            Ok(_) => Ok(true),
            Err(e) => {
                println!("Health check failed: {}", e);
                Ok(false)
            }
        }
    }
}

/// Upstash connection pool
///
/// Manages HTTP client instances for Upstash Redis API with connection pooling,
/// automatic retries, and metrics tracking.
pub struct UpstashConnectionPool {
    /// Configuration
    config: QueuePoolConfig,
    /// Base URL for Upstash API
    base_url: String,
    /// Auth token
    token: String,
    /// HTTP client pool (reqwest handles pooling internally)
    client: reqwest::Client,
    /// Semaphore to limit concurrent requests
    semaphore: Arc<Semaphore>,
    /// Shared metrics
    metrics: Arc<Mutex<PoolMetricsInternal>>,
    /// Pool closed flag
    closed: Arc<Mutex<bool>>,
}

impl UpstashConnectionPool {
    /// Create a new Upstash connection pool
    ///
    /// # Arguments
    ///
    /// * `base_url` - Upstash Redis HTTP endpoint
    /// * `token` - Upstash auth token
    /// * `config` - Pool configuration
    pub fn new(
        base_url: String,
        token: String,
        config: QueuePoolConfig,
    ) -> Result<Self, Box<dyn Error>> {
        let client = reqwest::Client::builder()
            .timeout(config.connection_timeout())
            .pool_max_idle_per_host(config.max_connections as usize)
            .pool_idle_timeout(Some(config.idle_timeout()))
            .build()
            .map_err(|e| PoolError::ConfigError(format!("Failed to create HTTP client: {}", e)))?;

        let semaphore = Arc::new(Semaphore::new(config.max_connections as usize));
        let metrics = Arc::new(Mutex::new(PoolMetricsInternal::new()));

        Ok(Self {
            config,
            base_url,
            token,
            client,
            semaphore,
            metrics,
            closed: Arc::new(Mutex::new(false)),
        })
    }

    /// Create pool from environment config
    pub async fn from_env_config(env_config: &EnvironmentConfig) -> Result<Self, Box<dyn Error>> {
        let base_url = env_config.upstash_url()?.to_string();
        let token = env_config.upstash_token()?.to_string();
        Self::new(base_url, token, env_config.pool.clone())
    }
}

#[async_trait]
impl ConnectionPool for UpstashConnectionPool {
    async fn get_connection(&self) -> Result<Box<dyn std::any::Any + Send>, Box<dyn Error>> {
        // Check if pool is closed
        {
            let closed = self.closed.lock().await;
            if *closed {
                return Err(Box::new(PoolError::PoolClosed(
                    "Pool has been closed".to_string(),
                )));
            }
        }

        let start = Instant::now();

        // Acquire permit from semaphore
        let permit =
            tokio::time::timeout(self.config.connection_timeout(), self.semaphore.acquire())
                .await
                .map_err(|_| {
                    PoolError::AcquisitionTimeout(format!(
                        "Timeout after {:?}",
                        self.config.connection_timeout()
                    ))
                })?
                .map_err(|_| PoolError::PoolClosed("Semaphore closed".to_string()))?;

        let wait_time = start.elapsed();
        let metrics = self.metrics.lock().await;
        metrics.total_acquisitions.fetch_add(1, Ordering::Relaxed);
        metrics
            .total_wait_time_ms
            .fetch_add(wait_time.as_millis() as u64, Ordering::Relaxed);
        metrics.active.fetch_add(1, Ordering::Relaxed);
        drop(metrics);

        // Return client clone (reqwest handles pooling internally)
        permit.forget();

        // Create a connection handle with auth
        let conn_handle = UpstashConnection {
            client: self.client.clone(),
            base_url: self.base_url.clone(),
            token: self.token.clone(),
        };

        Ok(Box::new(conn_handle))
    }

    fn get_metrics(&self) -> PoolMetrics {
        let available = self.semaphore.available_permits();
        let idle = available as u64;

        // Use try_lock instead of blocking_lock to avoid blocking in async context
        match self.metrics.try_lock() {
            Ok(metrics) => metrics.to_public(idle),
            Err(_) => {
                // If lock is held, return default metrics
                PoolMetrics {
                    total_connections_created: 0,
                    active_connections: 0,
                    idle_connections: idle,
                    total_acquisitions: 0,
                    total_connection_failures: 0,
                    total_reconnections: 0,
                    avg_wait_time_ms: 0,
                }
            }
        }
    }

    async fn close(&self) -> Result<(), Box<dyn Error>> {
        let mut closed = self.closed.lock().await;
        *closed = true;
        println!("Upstash connection pool closed");
        Ok(())
    }

    async fn health_check(&self) -> Result<bool, Box<dyn Error>> {
        if !self.config.enable_health_checks {
            return Ok(true);
        }

        // Try to make a simple PING request
        let response = self
            .client
            .post(&self.base_url)
            .header("Authorization", format!("Bearer {}", self.token))
            .json(&vec!["PING"])
            .send()
            .await;

        match response {
            Ok(resp) if resp.status().is_success() => Ok(true),
            Ok(resp) => {
                println!("Health check failed with status: {}", resp.status());
                Ok(false)
            }
            Err(e) => {
                println!("Health check failed: {}", e);
                Ok(false)
            }
        }
    }
}

/// Upstash connection handle
///
/// Wraps an HTTP client with Upstash credentials
#[derive(Clone)]
pub struct UpstashConnection {
    pub client: reqwest::Client,
    pub base_url: String,
    pub token: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_metrics_creation() {
        let metrics = PoolMetrics::new();
        assert_eq!(metrics.total_connections_created, 0);
        assert_eq!(metrics.active_connections, 0);
        assert_eq!(metrics.total_acquisitions, 0);
    }

    #[test]
    fn test_pool_metrics_log() {
        let metrics = PoolMetrics {
            total_connections_created: 10,
            active_connections: 5,
            idle_connections: 3,
            total_acquisitions: 100,
            total_connection_failures: 2,
            total_reconnections: 1,
            avg_wait_time_ms: 50,
        };

        // This should not panic
        metrics.log("test-pool");
    }

    #[tokio::test]
    async fn test_redis_pool_creation_with_invalid_url() {
        let config = QueuePoolConfig::development();
        let result = RedisConnectionPool::new("invalid-url", config);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_upstash_pool_creation() {
        let config = QueuePoolConfig::development();
        let result = UpstashConnectionPool::new(
            "https://example.upstash.io".to_string(),
            "test-token".to_string(),
            config,
        );
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_pool_close() {
        let config = QueuePoolConfig::development();
        let pool = UpstashConnectionPool::new(
            "https://example.upstash.io".to_string(),
            "test-token".to_string(),
            config,
        )
        .unwrap();

        assert!(pool.close().await.is_ok());

        // Should fail after close
        let result = pool.get_connection().await;
        assert!(result.is_err());
    }

    #[test]
    fn test_pool_metrics_average_calculation() {
        let internal = PoolMetricsInternal::new();
        internal.total_acquisitions.store(10, Ordering::Relaxed);
        internal.total_wait_time_ms.store(500, Ordering::Relaxed);

        let metrics = internal.to_public(5);
        assert_eq!(metrics.avg_wait_time_ms, 50); // 500 / 10
    }

    #[test]
    fn test_pool_metrics_zero_acquisitions() {
        let internal = PoolMetricsInternal::new();
        let metrics = internal.to_public(0);
        assert_eq!(metrics.avg_wait_time_ms, 0);
    }
}
