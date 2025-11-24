//! Environment-based configuration for MimiVibe backend
//!
//! This module provides configuration structures that are populated from environment
//! variables, with sensible defaults and validation. It supports different environments
//! (development, staging, production) with appropriate settings for each.

use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;
use std::time::Duration;

/// Environment enumeration
///
/// Represents the deployment environment of the application.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Environment {
    /// Development environment - relaxed limits, verbose logging
    Development,
    /// Staging environment - production-like settings with monitoring
    Staging,
    /// Production environment - strict limits, optimized settings
    Production,
}

impl Environment {
    /// Parse environment from string (use Environment::from_env() for standard FromStr)
    ///
    /// # Arguments
    ///
    /// * `s` - Environment string ("development", "staging", "production")
    ///
    /// # Returns
    ///
    /// The parsed environment, defaults to Development if unknown
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "production" | "prod" => Environment::Production,
            "staging" | "stage" => Environment::Staging,
            "development" | "dev" => Environment::Development,
            _ => {
                eprintln!("Unknown environment '{}', defaulting to Development", s);
                Environment::Development
            }
        }
    }

    /// Get environment from ENVIRONMENT variable
    ///
    /// Defaults to Development if not set or invalid
    pub fn from_env() -> Self {
        std::env::var("ENVIRONMENT")
            .map(|s| Self::from_str(&s))
            .unwrap_or(Environment::Development)
    }

    /// Check if this is a production environment
    pub fn is_production(&self) -> bool {
        matches!(self, Environment::Production)
    }

    /// Check if this is a development environment
    pub fn is_development(&self) -> bool {
        matches!(self, Environment::Development)
    }
}

impl fmt::Display for Environment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Environment::Development => write!(f, "development"),
            Environment::Staging => write!(f, "staging"),
            Environment::Production => write!(f, "production"),
        }
    }
}

/// Configuration error types
#[derive(Debug)]
pub enum ConfigError {
    /// Missing required environment variable
    MissingEnvVar(String),
    /// Invalid configuration value
    InvalidValue(String),
    /// Parse error
    ParseError(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConfigError::MissingEnvVar(var) => {
                write!(f, "Missing required environment variable: {}", var)
            }
            ConfigError::InvalidValue(msg) => write!(f, "Invalid configuration value: {}", msg),
            ConfigError::ParseError(msg) => write!(f, "Configuration parse error: {}", msg),
        }
    }
}

impl Error for ConfigError {}

/// Connection pool configuration
///
/// Defines the connection pool settings for Redis/Upstash queue backends.
/// Settings are environment-specific with appropriate defaults.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueuePoolConfig {
    /// Maximum number of connections in the pool
    pub max_connections: u32,
    /// Minimum number of idle connections to maintain
    pub min_idle_connections: u32,
    /// Connection timeout in seconds
    pub connection_timeout_secs: u64,
    /// Idle connection timeout in seconds (time before idle connection is closed)
    pub idle_timeout_secs: u64,
    /// Maximum lifetime of a connection in seconds (0 = no limit)
    pub max_lifetime_secs: u64,
    /// Number of retry attempts for failed connections
    pub max_retry_attempts: u32,
    /// Base delay for exponential backoff in milliseconds
    pub retry_backoff_base_ms: u64,
    /// Maximum backoff delay in milliseconds
    pub retry_backoff_max_ms: u64,
    /// Enable connection health checks
    pub enable_health_checks: bool,
    /// Health check interval in seconds
    pub health_check_interval_secs: u64,
}

impl QueuePoolConfig {
    /// Create pool config for development environment
    ///
    /// Relaxed limits, suitable for local development
    pub fn development() -> Self {
        Self {
            max_connections: 5,
            min_idle_connections: 1,
            connection_timeout_secs: 10,
            idle_timeout_secs: 300,  // 5 minutes
            max_lifetime_secs: 1800, // 30 minutes
            max_retry_attempts: 3,
            retry_backoff_base_ms: 100,
            retry_backoff_max_ms: 5000, // 5 seconds
            enable_health_checks: false,
            health_check_interval_secs: 60,
        }
    }

    /// Create pool config for staging environment
    ///
    /// Production-like settings with some relaxed limits
    pub fn staging() -> Self {
        Self {
            max_connections: 10,
            min_idle_connections: 2,
            connection_timeout_secs: 15,
            idle_timeout_secs: 600,  // 10 minutes
            max_lifetime_secs: 3600, // 1 hour
            max_retry_attempts: 5,
            retry_backoff_base_ms: 200,
            retry_backoff_max_ms: 10000, // 10 seconds
            enable_health_checks: true,
            health_check_interval_secs: 30,
        }
    }

    /// Create pool config for production environment
    ///
    /// Optimized settings for production workloads
    pub fn production() -> Self {
        Self {
            max_connections: 20,
            min_idle_connections: 5,
            connection_timeout_secs: 30,
            idle_timeout_secs: 900,  // 15 minutes
            max_lifetime_secs: 7200, // 2 hours
            max_retry_attempts: 5,
            retry_backoff_base_ms: 500,
            retry_backoff_max_ms: 30000, // 30 seconds
            enable_health_checks: true,
            health_check_interval_secs: 30,
        }
    }

    /// Load pool config from environment variables
    ///
    /// Falls back to environment-specific defaults if variables not set
    pub fn from_env(environment: Environment) -> Self {
        let base_config = match environment {
            Environment::Development => Self::development(),
            Environment::Staging => Self::staging(),
            Environment::Production => Self::production(),
        };

        Self {
            max_connections: Self::get_env_u32("POOL_MAX_CONNECTIONS")
                .unwrap_or(base_config.max_connections),
            min_idle_connections: Self::get_env_u32("POOL_MIN_IDLE_CONNECTIONS")
                .unwrap_or(base_config.min_idle_connections),
            connection_timeout_secs: Self::get_env_u64("POOL_CONNECTION_TIMEOUT_SECS")
                .unwrap_or(base_config.connection_timeout_secs),
            idle_timeout_secs: Self::get_env_u64("POOL_IDLE_TIMEOUT_SECS")
                .unwrap_or(base_config.idle_timeout_secs),
            max_lifetime_secs: Self::get_env_u64("POOL_MAX_LIFETIME_SECS")
                .unwrap_or(base_config.max_lifetime_secs),
            max_retry_attempts: Self::get_env_u32("POOL_MAX_RETRY_ATTEMPTS")
                .unwrap_or(base_config.max_retry_attempts),
            retry_backoff_base_ms: Self::get_env_u64("POOL_RETRY_BACKOFF_BASE_MS")
                .unwrap_or(base_config.retry_backoff_base_ms),
            retry_backoff_max_ms: Self::get_env_u64("POOL_RETRY_BACKOFF_MAX_MS")
                .unwrap_or(base_config.retry_backoff_max_ms),
            enable_health_checks: Self::get_env_bool("POOL_ENABLE_HEALTH_CHECKS")
                .unwrap_or(base_config.enable_health_checks),
            health_check_interval_secs: Self::get_env_u64("POOL_HEALTH_CHECK_INTERVAL_SECS")
                .unwrap_or(base_config.health_check_interval_secs),
        }
    }

    /// Validate configuration values
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Configuration is valid
    /// * `Err(ConfigError)` - Configuration has invalid values
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.max_connections == 0 {
            return Err(ConfigError::InvalidValue(
                "max_connections must be greater than 0".to_string(),
            ));
        }

        if self.min_idle_connections > self.max_connections {
            return Err(ConfigError::InvalidValue(
                "min_idle_connections cannot exceed max_connections".to_string(),
            ));
        }

        if self.connection_timeout_secs == 0 {
            return Err(ConfigError::InvalidValue(
                "connection_timeout_secs must be greater than 0".to_string(),
            ));
        }

        if self.max_retry_attempts == 0 {
            return Err(ConfigError::InvalidValue(
                "max_retry_attempts must be greater than 0".to_string(),
            ));
        }

        if self.retry_backoff_base_ms >= self.retry_backoff_max_ms {
            return Err(ConfigError::InvalidValue(
                "retry_backoff_base_ms must be less than retry_backoff_max_ms".to_string(),
            ));
        }

        Ok(())
    }

    /// Get connection timeout as Duration
    pub fn connection_timeout(&self) -> Duration {
        Duration::from_secs(self.connection_timeout_secs)
    }

    /// Get idle timeout as Duration
    pub fn idle_timeout(&self) -> Duration {
        Duration::from_secs(self.idle_timeout_secs)
    }

    /// Get max lifetime as Duration
    pub fn max_lifetime(&self) -> Duration {
        Duration::from_secs(self.max_lifetime_secs)
    }

    /// Get health check interval as Duration
    pub fn health_check_interval(&self) -> Duration {
        Duration::from_secs(self.health_check_interval_secs)
    }

    /// Get base backoff delay as Duration
    pub fn retry_backoff_base(&self) -> Duration {
        Duration::from_millis(self.retry_backoff_base_ms)
    }

    /// Get max backoff delay as Duration
    pub fn retry_backoff_max(&self) -> Duration {
        Duration::from_millis(self.retry_backoff_max_ms)
    }

    // Helper methods for parsing environment variables
    fn get_env_u32(key: &str) -> Option<u32> {
        std::env::var(key).ok().and_then(|v| v.parse::<u32>().ok())
    }

    fn get_env_u64(key: &str) -> Option<u64> {
        std::env::var(key).ok().and_then(|v| v.parse::<u64>().ok())
    }

    fn get_env_bool(key: &str) -> Option<bool> {
        std::env::var(key)
            .ok()
            .and_then(|v| match v.to_lowercase().as_str() {
                "true" | "1" | "yes" | "on" => Some(true),
                "false" | "0" | "no" | "off" => Some(false),
                _ => None,
            })
    }
}

/// Main environment configuration
///
/// Aggregates all configuration settings for the application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentConfig {
    /// Current environment
    pub environment: Environment,
    /// Queue connection pool configuration
    pub pool: QueuePoolConfig,
    /// Redis URL (if using Redis backend)
    pub redis_url: Option<String>,
    /// Upstash Redis URL (if using Upstash backend)
    pub upstash_url: Option<String>,
    /// Upstash Redis token (if using Upstash backend)
    pub upstash_token: Option<String>,
    /// Stream key for queue
    pub stream_key: String,
    /// Consumer group name
    pub consumer_group: String,
}

impl EnvironmentConfig {
    /// Load configuration from environment variables
    ///
    /// # Returns
    ///
    /// * `Ok(EnvironmentConfig)` - Successfully loaded configuration
    /// * `Err(ConfigError)` - Failed to load required configuration
    pub fn from_env() -> Result<Self, Box<dyn Error>> {
        let environment = Environment::from_env();
        let pool = QueuePoolConfig::from_env(environment);

        // Validate pool config
        pool.validate()?;

        let config = Self {
            environment,
            pool,
            redis_url: std::env::var("REDIS_URL").ok(),
            upstash_url: std::env::var("UPSTASH_REDIS_URL").ok(),
            upstash_token: std::env::var("UPSTASH_REDIS_TOKEN").ok(),
            stream_key: std::env::var("REDIS_STREAM_KEY")
                .or_else(|_| std::env::var("UPSTASH_REDIS_STREAM_KEY"))
                .unwrap_or_else(|_| "tarot:jobs".to_string()),
            consumer_group: std::env::var("REDIS_CONSUMER_GROUP")
                .or_else(|_| std::env::var("UPSTASH_REDIS_CONSUMER_GROUP"))
                .unwrap_or_else(|_| "tarot-workers".to_string()),
        };

        // Validate that at least one queue backend is configured
        if config.redis_url.is_none()
            && (config.upstash_url.is_none() || config.upstash_token.is_none())
        {
            return Err(Box::new(ConfigError::MissingEnvVar(
                "Either REDIS_URL or (UPSTASH_REDIS_URL + UPSTASH_REDIS_TOKEN) must be set"
                    .to_string(),
            )));
        }

        println!(
            "Loaded configuration for {} environment",
            config.environment
        );
        println!("  Pool max connections: {}", config.pool.max_connections);
        println!("  Pool min idle: {}", config.pool.min_idle_connections);
        println!(
            "  Connection timeout: {}s",
            config.pool.connection_timeout_secs
        );
        println!("  Health checks: {}", config.pool.enable_health_checks);

        Ok(config)
    }

    /// Check if Redis backend is configured
    pub fn has_redis(&self) -> bool {
        self.redis_url.is_some()
    }

    /// Check if Upstash backend is configured
    pub fn has_upstash(&self) -> bool {
        self.upstash_url.is_some() && self.upstash_token.is_some()
    }

    /// Get Redis URL or error
    pub fn redis_url(&self) -> Result<&str, ConfigError> {
        self.redis_url
            .as_deref()
            .ok_or_else(|| ConfigError::MissingEnvVar("REDIS_URL".to_string()))
    }

    /// Get Upstash URL or error
    pub fn upstash_url(&self) -> Result<&str, ConfigError> {
        self.upstash_url
            .as_deref()
            .ok_or_else(|| ConfigError::MissingEnvVar("UPSTASH_REDIS_URL".to_string()))
    }

    /// Get Upstash token or error
    pub fn upstash_token(&self) -> Result<&str, ConfigError> {
        self.upstash_token
            .as_deref()
            .ok_or_else(|| ConfigError::MissingEnvVar("UPSTASH_REDIS_TOKEN".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_parsing() {
        assert_eq!(Environment::from_str("production"), Environment::Production);
        assert_eq!(Environment::from_str("PRODUCTION"), Environment::Production);
        assert_eq!(Environment::from_str("prod"), Environment::Production);
        assert_eq!(Environment::from_str("staging"), Environment::Staging);
        assert_eq!(
            Environment::from_str("development"),
            Environment::Development
        );
        assert_eq!(Environment::from_str("dev"), Environment::Development);
        assert_eq!(Environment::from_str("unknown"), Environment::Development);
    }

    #[test]
    fn test_environment_checks() {
        assert!(Environment::Production.is_production());
        assert!(!Environment::Production.is_development());
        assert!(Environment::Development.is_development());
        assert!(!Environment::Development.is_production());
    }

    #[test]
    fn test_pool_config_development() {
        let config = QueuePoolConfig::development();
        assert_eq!(config.max_connections, 5);
        assert_eq!(config.min_idle_connections, 1);
        assert!(!config.enable_health_checks);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_pool_config_staging() {
        let config = QueuePoolConfig::staging();
        assert_eq!(config.max_connections, 10);
        assert_eq!(config.min_idle_connections, 2);
        assert!(config.enable_health_checks);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_pool_config_production() {
        let config = QueuePoolConfig::production();
        assert_eq!(config.max_connections, 20);
        assert_eq!(config.min_idle_connections, 5);
        assert!(config.enable_health_checks);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_pool_config_validation_invalid_max_connections() {
        let mut config = QueuePoolConfig::development();
        config.max_connections = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_pool_config_validation_invalid_min_idle() {
        let mut config = QueuePoolConfig::development();
        config.min_idle_connections = 10;
        config.max_connections = 5;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_pool_config_validation_invalid_timeout() {
        let mut config = QueuePoolConfig::development();
        config.connection_timeout_secs = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_pool_config_validation_invalid_backoff() {
        let mut config = QueuePoolConfig::development();
        config.retry_backoff_base_ms = 5000;
        config.retry_backoff_max_ms = 1000;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_pool_config_duration_conversions() {
        let config = QueuePoolConfig::development();
        assert_eq!(config.connection_timeout(), Duration::from_secs(10));
        assert_eq!(config.idle_timeout(), Duration::from_secs(300));
        assert_eq!(config.retry_backoff_base(), Duration::from_millis(100));
    }
}
