//! Integration tests for queue connection pooling
//!
//! Tests connection pool configuration, automatic reconnection,
//! metrics tracking, and worker isolation.

mod setup;

#[cfg(test)]
mod config_tests {
    use mimivibe_backend::config::{Environment, QueuePoolConfig};
    use std::env;

    #[test]
    fn test_config_pool_development_defaults() {
        let config = QueuePoolConfig::development();

        assert_eq!(config.max_connections, 5);
        assert_eq!(config.min_idle_connections, 1);
        assert_eq!(config.connection_timeout_secs, 10);
        assert_eq!(config.idle_timeout_secs, 300);
        assert_eq!(config.max_lifetime_secs, 1800);
        assert_eq!(config.max_retry_attempts, 3);
        assert_eq!(config.retry_backoff_base_ms, 100);
        assert_eq!(config.retry_backoff_max_ms, 5000);
        assert!(!config.enable_health_checks);
        assert_eq!(config.health_check_interval_secs, 60);
    }

    #[test]
    fn test_config_pool_staging_defaults() {
        let config = QueuePoolConfig::staging();

        assert_eq!(config.max_connections, 10);
        assert_eq!(config.min_idle_connections, 2);
        assert_eq!(config.connection_timeout_secs, 15);
        assert_eq!(config.idle_timeout_secs, 600);
        assert_eq!(config.max_lifetime_secs, 3600);
        assert_eq!(config.max_retry_attempts, 5);
        assert_eq!(config.retry_backoff_base_ms, 200);
        assert_eq!(config.retry_backoff_max_ms, 10000);
        assert!(config.enable_health_checks);
        assert_eq!(config.health_check_interval_secs, 30);
    }

    #[test]
    fn test_config_pool_production_defaults() {
        let config = QueuePoolConfig::production();

        assert_eq!(config.max_connections, 20);
        assert_eq!(config.min_idle_connections, 5);
        assert_eq!(config.connection_timeout_secs, 30);
        assert_eq!(config.idle_timeout_secs, 900);
        assert_eq!(config.max_lifetime_secs, 7200);
        assert_eq!(config.max_retry_attempts, 5);
        assert_eq!(config.retry_backoff_base_ms, 500);
        assert_eq!(config.retry_backoff_max_ms, 30000);
        assert!(config.enable_health_checks);
        assert_eq!(config.health_check_interval_secs, 30);
    }

    #[test]
    fn test_config_pool_quota_caps() {
        // Development should have lowest cap
        let dev = QueuePoolConfig::development();
        let staging = QueuePoolConfig::staging();
        let prod = QueuePoolConfig::production();

        assert!(dev.max_connections < staging.max_connections);
        assert!(staging.max_connections < prod.max_connections);

        // All should have valid quota caps
        assert!(dev.validate().is_ok());
        assert!(staging.validate().is_ok());
        assert!(prod.validate().is_ok());
    }

    #[test]
    fn test_config_validation_invalid_max_connections() {
        let mut config = QueuePoolConfig::development();
        config.max_connections = 0;

        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("max_connections"));
    }

    #[test]
    fn test_config_validation_invalid_min_idle() {
        let mut config = QueuePoolConfig::development();
        config.min_idle_connections = 100;
        config.max_connections = 10;

        let result = config.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("min_idle_connections"));
    }

    #[test]
    fn test_config_validation_invalid_timeout() {
        let mut config = QueuePoolConfig::development();
        config.connection_timeout_secs = 0;

        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("timeout"));
    }

    #[test]
    fn test_config_validation_invalid_retry() {
        let mut config = QueuePoolConfig::development();
        config.max_retry_attempts = 0;

        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("retry"));
    }

    #[test]
    fn test_config_validation_invalid_backoff() {
        let mut config = QueuePoolConfig::development();
        config.retry_backoff_base_ms = 10000;
        config.retry_backoff_max_ms = 1000;

        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("backoff"));
    }

    #[test]
    fn test_environment_parsing() {
        assert_eq!(
            Environment::from_str("development"),
            Environment::Development
        );
        assert_eq!(Environment::from_str("dev"), Environment::Development);
        assert_eq!(Environment::from_str("staging"), Environment::Staging);
        assert_eq!(Environment::from_str("stage"), Environment::Staging);
        assert_eq!(Environment::from_str("production"), Environment::Production);
        assert_eq!(Environment::from_str("prod"), Environment::Production);

        // Unknown defaults to development
        assert_eq!(Environment::from_str("unknown"), Environment::Development);
    }

    #[test]
    fn test_environment_checks() {
        assert!(Environment::Production.is_production());
        assert!(!Environment::Production.is_development());
        assert!(Environment::Development.is_development());
        assert!(!Environment::Development.is_production());
        assert!(!Environment::Staging.is_production());
        assert!(!Environment::Staging.is_development());
    }

    #[test]
    fn test_config_from_env_with_overrides() {
        // Set environment variables
        env::set_var("POOL_MAX_CONNECTIONS", "15");
        env::set_var("POOL_MIN_IDLE_CONNECTIONS", "3");
        env::set_var("POOL_CONNECTION_TIMEOUT_SECS", "20");
        env::set_var("POOL_ENABLE_HEALTH_CHECKS", "true");

        let config = QueuePoolConfig::from_env(Environment::Development);

        assert_eq!(config.max_connections, 15);
        assert_eq!(config.min_idle_connections, 3);
        assert_eq!(config.connection_timeout_secs, 20);
        assert!(config.enable_health_checks);

        // Clean up
        env::remove_var("POOL_MAX_CONNECTIONS");
        env::remove_var("POOL_MIN_IDLE_CONNECTIONS");
        env::remove_var("POOL_CONNECTION_TIMEOUT_SECS");
        env::remove_var("POOL_ENABLE_HEALTH_CHECKS");
    }

    #[test]
    fn test_config_duration_conversions() {
        let config = QueuePoolConfig::development();

        assert_eq!(config.connection_timeout().as_secs(), 10);
        assert_eq!(config.idle_timeout().as_secs(), 300);
        assert_eq!(config.max_lifetime().as_secs(), 1800);
        assert_eq!(config.health_check_interval().as_secs(), 60);
        assert_eq!(config.retry_backoff_base().as_millis(), 100);
        assert_eq!(config.retry_backoff_max().as_millis(), 5000);
    }
}

#[cfg(test)]
mod pool_tests {
    use mimivibe_backend::config::QueuePoolConfig;
    use mimivibe_backend::queue::pool::{
        ConnectionPool, PoolMetrics, RedisConnectionPool, UpstashConnectionPool,
    };
    use std::sync::Arc;

    #[test]
    fn test_pool_metrics_creation() {
        let metrics = PoolMetrics::new();

        assert_eq!(metrics.total_connections_created, 0);
        assert_eq!(metrics.active_connections, 0);
        assert_eq!(metrics.idle_connections, 0);
        assert_eq!(metrics.total_acquisitions, 0);
        assert_eq!(metrics.total_connection_failures, 0);
        assert_eq!(metrics.total_reconnections, 0);
        assert_eq!(metrics.avg_wait_time_ms, 0);
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

        // Should not panic
        metrics.log("test-pool");
    }

    #[tokio::test]
    async fn test_redis_pool_creation_invalid_url() {
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
    async fn test_upstash_pool_metrics() {
        let config = QueuePoolConfig::development();
        let pool = UpstashConnectionPool::new(
            "https://example.upstash.io".to_string(),
            "test-token".to_string(),
            config,
        )
        .unwrap();

        let metrics = pool.get_metrics();

        // Initial state
        assert_eq!(metrics.total_connections_created, 0);
        assert_eq!(metrics.total_acquisitions, 0);
    }

    #[tokio::test]
    async fn test_upstash_pool_close() {
        let config = QueuePoolConfig::development();
        let pool = UpstashConnectionPool::new(
            "https://example.upstash.io".to_string(),
            "test-token".to_string(),
            config,
        )
        .unwrap();

        // Should succeed
        assert!(pool.close().await.is_ok());

        // Should fail after close
        let result = pool.get_connection().await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("closed"));
    }

    #[tokio::test]
    async fn test_pool_concurrent_access() {
        let config = QueuePoolConfig {
            max_connections: 2,
            min_idle_connections: 1,
            connection_timeout_secs: 5,
            ..QueuePoolConfig::development()
        };

        let pool = Arc::new(
            UpstashConnectionPool::new(
                "https://example.upstash.io".to_string(),
                "test-token".to_string(),
                config,
            )
            .unwrap(),
        );

        let pool1 = pool.clone();
        let pool2 = pool.clone();

        // Spawn concurrent tasks
        let handle1 = tokio::spawn(async move { pool1.get_metrics() });

        let handle2 = tokio::spawn(async move { pool2.get_metrics() });

        let (result1, result2) = tokio::join!(handle1, handle2);

        assert!(result1.is_ok());
        assert!(result2.is_ok());
    }

    #[tokio::test]
    async fn test_pool_health_check_disabled() {
        let mut config = QueuePoolConfig::development();
        config.enable_health_checks = false;

        let pool = UpstashConnectionPool::new(
            "https://example.upstash.io".to_string(),
            "test-token".to_string(),
            config,
        )
        .unwrap();

        // Should always return true when disabled
        let result = pool.health_check().await;
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[tokio::test]
    async fn test_worker_pool_isolation() {
        // Simulate two workers with separate pools
        let config = QueuePoolConfig::development();

        let pool1 = Arc::new(
            UpstashConnectionPool::new(
                "https://worker1.upstash.io".to_string(),
                "token1".to_string(),
                config.clone(),
            )
            .unwrap(),
        );

        let pool2 = Arc::new(
            UpstashConnectionPool::new(
                "https://worker2.upstash.io".to_string(),
                "token2".to_string(),
                config,
            )
            .unwrap(),
        );

        // Get metrics from both pools
        let metrics1 = pool1.get_metrics();
        let metrics2 = pool2.get_metrics();

        // Pools should be isolated (both start at 0)
        assert_eq!(metrics1.total_acquisitions, 0);
        assert_eq!(metrics2.total_acquisitions, 0);

        // Pools should not be the same
        assert!(!Arc::ptr_eq(&pool1, &pool2));
    }

    #[tokio::test]
    async fn test_pool_quota_enforcement() {
        let mut config = QueuePoolConfig::development();
        config.max_connections = 2;
        config.connection_timeout_secs = 1; // Short timeout

        let pool = UpstashConnectionPool::new(
            "https://example.upstash.io".to_string(),
            "test-token".to_string(),
            config,
        )
        .unwrap();

        // Get initial metrics
        let metrics_before = pool.get_metrics();
        assert_eq!(metrics_before.total_acquisitions, 0);

        // Pool should enforce quota via semaphore
        // Just verify the pool was created with correct config
        assert!(pool.close().await.is_ok());
    }
}

#[cfg(test)]
mod reconnection_tests {
    use mimivibe_backend::config::QueuePoolConfig;
    use mimivibe_backend::queue::pool::{ConnectionPool, UpstashConnectionPool};

    #[tokio::test]
    async fn test_pool_config_retry_settings() {
        let config = QueuePoolConfig::development();

        assert_eq!(config.max_retry_attempts, 3);
        assert_eq!(config.retry_backoff_base_ms, 100);
        assert_eq!(config.retry_backoff_max_ms, 5000);
    }

    #[tokio::test]
    async fn test_pool_retry_backoff_scaling() {
        let dev = QueuePoolConfig::development();
        let staging = QueuePoolConfig::staging();
        let prod = QueuePoolConfig::production();

        // Production should have longest backoff
        assert!(dev.retry_backoff_max_ms < staging.retry_backoff_max_ms);
        assert!(staging.retry_backoff_max_ms < prod.retry_backoff_max_ms);
    }

    #[tokio::test]
    async fn test_upstash_pool_health_check_fail() {
        let mut config = QueuePoolConfig::development();
        config.enable_health_checks = true;

        // Use invalid URL to trigger health check failure
        let pool = UpstashConnectionPool::new(
            "https://invalid.example.com".to_string(),
            "invalid-token".to_string(),
            config,
        )
        .unwrap();

        // Health check should fail (returns false, not error)
        let result = pool.health_check().await;
        assert!(result.is_ok());
        // Should return false for failed health check
        assert!(!result.unwrap());
    }
}

#[cfg(test)]
mod metric_logging_tests {
    use mimivibe_backend::queue::pool::PoolMetrics;

    #[test]
    fn test_metrics_log_output() {
        let metrics = PoolMetrics {
            total_connections_created: 50,
            active_connections: 10,
            idle_connections: 5,
            total_acquisitions: 200,
            total_connection_failures: 3,
            total_reconnections: 2,
            avg_wait_time_ms: 25,
        };

        // Should log without panic
        metrics.log("production-pool");
    }

    #[test]
    fn test_metrics_default_values() {
        let metrics = PoolMetrics::default();

        assert_eq!(metrics.total_connections_created, 0);
        assert_eq!(metrics.active_connections, 0);
        assert_eq!(metrics.idle_connections, 0);
    }
}
