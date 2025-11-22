//! Retry & Exponential Backoff Implementation for Worker
//!
//! Provides robust retry logic with exponential backoff and jitter
//! to handle temporary failures in job processing.

use crate::queue::QueuedJob;
use std::error::Error;
use std::fmt;
use std::time::Duration;

/// Error types for retry policy
#[derive(Debug)]
pub enum RetryError {
    /// Invalid configuration
    InvalidConfig(String),
    /// Retry attempts exceeded
    MaxAttemptsExceeded,
}

impl fmt::Display for RetryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RetryError::InvalidConfig(msg) => write!(f, "Invalid retry config: {}", msg),
            RetryError::MaxAttemptsExceeded => write!(f, "Maximum retry attempts exceeded"),
        }
    }
}

impl Error for RetryError {}

/// Configuration for retry policy
#[derive(Debug, Clone, Copy)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Base delay between retries
    pub base_delay: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Multiplier for exponential backoff
    pub backoff_multiplier: f64,
    /// Whether to add jitter to delays
    pub jitter: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(1000),
            max_delay: Duration::from_millis(30000),
            backoff_multiplier: 2.0,
            jitter: true,
        }
    }
}

/// Retry policy for handling job failures
#[derive(Debug)]
pub struct RetryPolicy {
    pub config: RetryConfig,
}

impl RetryPolicy {
    /// Create a new retry policy with the given configuration
    ///
    /// # Arguments
    /// * `config` - Retry configuration with validated parameters
    ///
    /// # Returns
    /// * `Ok(RetryPolicy)` - Valid retry policy
    /// * `Err(RetryError)` - Configuration validation error
    ///
    /// # Example
    /// ```rust
    /// let config = RetryConfig::default();
    /// let policy = RetryPolicy::new(config)?;
    /// ```
    pub fn new(config: RetryConfig) -> Result<Self, RetryError> {
        Self::validate_config(&config)?;
        Ok(Self { config })
    }

    /// Validate retry configuration parameters
    fn validate_config(config: &RetryConfig) -> Result<(), RetryError> {
        if config.max_attempts == 0 {
            return Err(RetryError::InvalidConfig(
                "max_attempts must be > 0".to_string(),
            ));
        }
        if config.base_delay == Duration::from_millis(0) {
            return Err(RetryError::InvalidConfig(
                "base_delay must be > 0".to_string(),
            ));
        }
        if config.max_delay <= config.base_delay {
            return Err(RetryError::InvalidConfig(
                "max_delay must be > base_delay".to_string(),
            ));
        }
        if config.backoff_multiplier <= 1.0 {
            return Err(RetryError::InvalidConfig(
                "backoff_multiplier must be > 1.0".to_string(),
            ));
        }
        Ok(())
    }

    /// Determine if a job should be retried based on attempts and error
    pub fn should_retry(&self, attempts: u32, _error: &(dyn Error + Send + Sync)) -> bool {
        attempts < self.config.max_attempts
    }

    /// Calculate delay for the next retry attempt
    ///
    /// Uses exponential backoff: delay = base_delay * (multiplier ^ (attempts-1))
    /// Capped at max_delay to prevent excessive delays
    /// Applies jitter if enabled to prevent thundering herd
    ///
    /// # Arguments
    /// * `attempts` - Current attempt number (1-based)
    ///
    /// # Returns
    /// * `Duration` - Calculated delay before next retry
    pub fn calculate_delay(&self, attempts: u32) -> Duration {
        let base_delay = self.config.base_delay;

        // Calculate exponential backoff with overflow protection
        let exponent = attempts.saturating_sub(1);
        let multiplier = self.config.backoff_multiplier.powi(exponent as i32);

        // Use f64 for calculation to handle large numbers, then convert back
        let delay_millis = base_delay.as_millis() as f64 * multiplier;

        // Cap at max_delay to prevent excessive delays
        let capped_delay_millis = delay_millis.min(self.config.max_delay.as_millis() as f64);

        // Convert to Duration, ensuring no overflow
        let delay = Duration::from_millis(capped_delay_millis as u64);

        if self.config.jitter {
            self.add_jitter(delay)
        } else {
            delay
        }
    }

    /// Add jitter to delay to prevent thundering herd
    ///
    /// Implements full jitter strategy: random delay between 0 and calculated delay
    /// This prevents multiple workers from retrying simultaneously (thundering herd)
    fn add_jitter(&self, delay: Duration) -> Duration {
        use rand::Rng;

        // Full jitter: random value between 0 and delay
        let jittered_millis = rand::thread_rng().gen_range(0.0..=delay.as_millis() as f64);

        // Ensure minimum delay of at least base_delay / 4 to prevent too rapid retries
        let min_delay = (self.config.base_delay.as_millis() as f64 * 0.25).max(1.0);
        let final_delay = jittered_millis.max(min_delay);

        Duration::from_millis(final_delay as u64)
    }

    /// Get delay for next attempt, or None if max attempts reached
    pub fn next_attempt_delay(
        &self,
        job: &QueuedJob,
        error: &(dyn Error + Send + Sync),
    ) -> Option<Duration> {
        if self.should_retry(job.attempts, error) {
            Some(self.calculate_delay(job.attempts + 1))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::queue::{JobPayload, QueuedJob};
    use chrono::Utc;
    use serde_json::json;
    use uuid::Uuid;

    fn create_test_queued_job(attempts: u32) -> QueuedJob {
        QueuedJob {
            job_id: Uuid::new_v4().to_string(),
            payload: JobPayload {
                job_id: Uuid::new_v4().to_string(),
                user_id: Uuid::new_v4(),
                question: "Test question".to_string(),
                card_count: 3,
                schema_version: "1".to_string(),
                prompt_version: "v2025-11-20-a".to_string(),
                dedupe_key: None,
                trace_id: None,
                created_at: Utc::now(),
                metadata: json!({}),
            },
            attempts,
            claimed_at: Utc::now(),
        }
    }

    fn mock_error() -> Box<dyn Error + Send + Sync> {
        Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Mock error"))
    }

    #[test]
    fn test_valid_config_creation() {
        let config = RetryConfig {
            max_attempts: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_millis(5000),
            backoff_multiplier: 2.0,
            jitter: true,
        };

        let policy = RetryPolicy::new(config);
        assert!(policy.is_ok());
    }

    #[test]
    fn test_invalid_config_rejection() {
        let invalid_configs = vec![
            RetryConfig {
                max_attempts: 0,
                base_delay: Duration::from_millis(100),
                max_delay: Duration::from_millis(5000),
                backoff_multiplier: 2.0,
                jitter: false,
            },
            RetryConfig {
                max_attempts: 3,
                base_delay: Duration::from_millis(0),
                max_delay: Duration::from_millis(5000),
                backoff_multiplier: 2.0,
                jitter: false,
            },
            RetryConfig {
                max_attempts: 3,
                base_delay: Duration::from_millis(100),
                max_delay: Duration::from_millis(50),
                backoff_multiplier: 2.0,
                jitter: false,
            },
            RetryConfig {
                max_attempts: 3,
                base_delay: Duration::from_millis(100),
                max_delay: Duration::from_millis(5000),
                backoff_multiplier: 1.0,
                jitter: false,
            },
        ];

        for config in invalid_configs {
            let policy = RetryPolicy::new(config);
            assert!(policy.is_err());
        }
    }

    #[test]
    fn test_should_retry_logic() {
        let config = RetryConfig::default();
        let policy = RetryPolicy::new(config).unwrap();

        assert!(policy.should_retry(1, mock_error().as_ref()));
        assert!(policy.should_retry(2, mock_error().as_ref()));
        assert!(!policy.should_retry(3, mock_error().as_ref()));
        assert!(!policy.should_retry(4, mock_error().as_ref()));
    }

    #[test]
    fn test_exponential_backoff_without_jitter() {
        let config = RetryConfig {
            max_attempts: 5,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_millis(5000),
            backoff_multiplier: 2.0,
            jitter: false,
        };

        let policy = RetryPolicy::new(config).unwrap();

        // Test exponential backoff
        let expected_delays = [
            Duration::from_millis(100),  // 100 * 2^0
            Duration::from_millis(200),  // 100 * 2^1
            Duration::from_millis(400),  // 100 * 2^2
            Duration::from_millis(800),  // 100 * 2^3
            Duration::from_millis(1600), // 100 * 2^4
        ];

        for (i, expected_delay) in expected_delays.iter().enumerate() {
            let actual_delay = policy.calculate_delay((i + 1) as u32);
            assert_eq!(actual_delay, *expected_delay);
        }
    }

    #[test]
    fn test_delay_capping() {
        let config = RetryConfig {
            max_attempts: 10,
            base_delay: Duration::from_millis(1000),
            max_delay: Duration::from_millis(3000),
            backoff_multiplier: 3.0,
            jitter: false,
        };

        let max_delay = config.max_delay; // Save before moving
        let policy = RetryPolicy::new(config).unwrap();

        for attempt in 1..=10 {
            let delay = policy.calculate_delay(attempt);
            assert!(delay <= max_delay);
        }
    }

    #[test]
    fn test_jitter_variation() {
        let config = RetryConfig {
            max_attempts: 3,
            base_delay: Duration::from_millis(1000),
            max_delay: Duration::from_millis(10000),
            backoff_multiplier: 2.0,
            jitter: true,
        };

        let policy = RetryPolicy::new(config).unwrap();

        let mut delays = Vec::new();
        for _ in 0..20 {
            delays.push(policy.calculate_delay(1));
        }

        // Should have variation due to jitter (with high probability)
        let unique_delays: std::collections::HashSet<_> = delays.iter().collect();
        assert!(
            unique_delays.len() > 1,
            "Jitter should create variation, but got {} identical values",
            unique_delays.len()
        );
    }

    #[test]
    fn test_next_attempt_delay() {
        let config = RetryConfig {
            max_attempts: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_millis(1000),
            backoff_multiplier: 2.0,
            jitter: false,
        };

        let policy = RetryPolicy::new(config).unwrap();

        // Job with 1 attempt should get retry delay
        let job_with_attempts = create_test_queued_job(1);
        let next_delay = policy.next_attempt_delay(&job_with_attempts, mock_error().as_ref());
        assert!(next_delay.is_some());
        assert_eq!(next_delay.unwrap(), Duration::from_millis(200));

        // Job with 3 attempts should get None
        let job_at_max = create_test_queued_job(3);
        let no_delay = policy.next_attempt_delay(&job_at_max, mock_error().as_ref());
        assert!(no_delay.is_none());
    }
}
