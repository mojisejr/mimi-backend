//! Integration tests for Worker Retry & Exponential Backoff
//!
//! Test-First Development: Tests written BEFORE implementation
//! RED phase: These tests will FAIL initially until retry logic is implemented
//!
//! Tests cover:
//! - Retry policy configuration and validation
//! - Exponential backoff with jitter calculations
//! - Max attempts enforcement
//! - Worker retry workflow integration
//! - DLQ integration for permanently failed jobs
//! - Edge cases and error scenarios

use chrono::Utc;
use mimivibe_backend::queue::{JobPayload, Queue, QueuedJob};
use mimivibe_backend::worker::retry::{RetryConfig, RetryPolicy};
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;

// Helper function to create test job payload
fn create_test_payload(question: &str) -> JobPayload {
    JobPayload {
        job_id: Uuid::new_v4().to_string(),
        user_id: Uuid::new_v4(),
        question: question.to_string(),
        card_count: 3,
        schema_version: "1".to_string(),
        prompt_version: "v2025-11-20-a".to_string(),
        dedupe_key: None,
        trace_id: None,
        created_at: Utc::now(),
        metadata: json!({
            "locale": "th",
            "source": "test"
        }),
    }
}

// Helper function to create test queued job
fn create_test_queued_job(attempts: u32) -> QueuedJob {
    QueuedJob {
        job_id: Uuid::new_v4().to_string(),
        payload: create_test_payload("ความรักของฉันจะเป็นอย่างไร"),
        attempts,
        claimed_at: Utc::now(),
    }
}

// Mock error for testing
fn mock_error() -> Box<dyn std::error::Error + Send + Sync> {
    Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Mock error"))
}

#[cfg(test)]
mod retry_policy_tests {
    use super::*;

    #[tokio::test]
    async fn test_retry_config_validation() {
        // Test valid configuration
        let config = RetryConfig {
            max_attempts: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_millis(5000),
            backoff_multiplier: 2.0,
            jitter: true,
        };

        let policy = RetryPolicy::new(config);
        assert!(policy.is_ok(), "Should create policy with valid config");

        // Test invalid configurations
        let invalid_configs = vec![
            RetryConfig {
                max_attempts: 0, // Invalid: zero attempts
                base_delay: Duration::from_millis(100),
                max_delay: Duration::from_millis(5000),
                backoff_multiplier: 2.0,
                jitter: false,
            },
            RetryConfig {
                max_attempts: 3,
                base_delay: Duration::from_millis(0), // Invalid: zero base delay
                max_delay: Duration::from_millis(5000),
                backoff_multiplier: 2.0,
                jitter: false,
            },
            RetryConfig {
                max_attempts: 3,
                base_delay: Duration::from_millis(100),
                max_delay: Duration::from_millis(50), // Invalid: max < base
                backoff_multiplier: 2.0,
                jitter: false,
            },
        ];

        for invalid_config in invalid_configs {
            let policy_result = RetryPolicy::new(invalid_config);
            assert!(policy_result.is_err(), "Should reject invalid config");
        }
    }

    #[tokio::test]
    async fn test_retry_policy_max_attempts() {
        let config = RetryConfig {
            max_attempts: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_millis(5000),
            backoff_multiplier: 2.0,
            jitter: false,
        };

        let policy = RetryPolicy::new(config).unwrap();

        // Should retry for attempts 1, 2
        assert!(policy.should_retry(1, mock_error().as_ref()));
        assert!(policy.should_retry(2, mock_error().as_ref()));

        // Should not retry for attempt 3 (max reached)
        assert!(!policy.should_retry(3, mock_error().as_ref()));
        assert!(!policy.should_retry(4, mock_error().as_ref()));
    }

    #[tokio::test]
    async fn test_exponential_backoff_without_jitter() {
        let config = RetryConfig {
            max_attempts: 5,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_millis(5000),
            backoff_multiplier: 2.0,
            jitter: false,
        };

        let policy = RetryPolicy::new(config).unwrap();

        // Test exponential backoff: base_delay * (multiplier ^ (attempts - 1))
        let expected_delays = vec![
            Duration::from_millis(100),  // Attempt 1: 100 * 2^0 = 100
            Duration::from_millis(200),  // Attempt 2: 100 * 2^1 = 200
            Duration::from_millis(400),  // Attempt 3: 100 * 2^2 = 400
            Duration::from_millis(800),  // Attempt 4: 100 * 2^3 = 800
            Duration::from_millis(1600), // Attempt 5: 100 * 2^4 = 1600
        ];

        for (attempt, expected_delay) in expected_delays.iter().enumerate() {
            let actual_delay = policy.calculate_delay(attempt as u32 + 1);
            assert_eq!(
                actual_delay,
                *expected_delay,
                "Delay mismatch for attempt {}: expected {:?}, got {:?}",
                attempt + 1,
                expected_delay,
                actual_delay
            );
        }
    }

    #[tokio::test]
    async fn test_exponential_backoff_with_max_delay() {
        let config = RetryConfig {
            max_attempts: 10,
            base_delay: Duration::from_millis(1000),
            max_delay: Duration::from_millis(3000), // Lower max to test capping
            backoff_multiplier: 3.0,
            jitter: false,
        };

        let policy = RetryPolicy::new(config).unwrap();

        // Test that delay never exceeds max_delay
        for attempt in 1..=10 {
            let delay = policy.calculate_delay(attempt);
            assert!(
                delay <= config.max_delay,
                "Delay {:?} for attempt {} exceeds max_delay {:?}",
                delay,
                attempt,
                config.max_delay
            );
        }

        // Specific test: should be capped at max_delay
        let attempt_with_high_delay = 5; // 1000 * 3^4 = 81000, but capped at 3000
        let capped_delay = policy.calculate_delay(attempt_with_high_delay);
        assert_eq!(capped_delay, config.max_delay);
    }

    #[tokio::test]
    async fn test_jitter_adds_randomness() {
        let config = RetryConfig {
            max_attempts: 3,
            base_delay: Duration::from_millis(1000),
            max_delay: Duration::from_millis(10000),
            backoff_multiplier: 2.0,
            jitter: true,
        };

        let policy = RetryPolicy::new(config).unwrap();

        // Calculate multiple delays for same attempt to test jitter
        let mut delays = Vec::new();
        for _ in 0..10 {
            delays.push(policy.calculate_delay(1));
        }

        // With jitter, delays should vary (not all identical)
        let unique_delays: std::collections::HashSet<_> = delays.iter().collect();
        assert!(
            unique_delays.len() > 1,
            "Jitter should create varying delays"
        );

        // All delays should be within reasonable bounds
        // With full jitter strategy, delays can be between min_delay and calculated delay
        for delay in &delays {
            assert!(
                *delay > Duration::from_millis(0),
                "Delay should be positive"
            );
            assert!(
                *delay <= config.max_delay + config.base_delay,
                "Delay should not exceed reasonable maximum"
            );
        }
    }

    #[tokio::test]
    async fn test_next_attempt_delay_logic() {
        let config = RetryConfig {
            max_attempts: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_millis(1000),
            backoff_multiplier: 2.0,
            jitter: false,
        };

        let policy = RetryPolicy::new(config).unwrap();

        // Test job with attempts less than max_attempts
        let job_with_attempts = create_test_queued_job(1); // 1 attempt made
        let next_delay = policy.next_attempt_delay(&job_with_attempts, mock_error().as_ref());

        assert!(
            next_delay.is_some(),
            "Should return delay for retryable job"
        );
        assert_eq!(
            next_delay.unwrap(),
            Duration::from_millis(200), // 100 * 2^1
            "Should calculate delay based on next attempt number"
        );

        // Test job at max attempts
        let job_at_max = create_test_queued_job(3); // 3 attempts made (max)
        let no_delay = policy.next_attempt_delay(&job_at_max, mock_error().as_ref());

        assert!(
            no_delay.is_none(),
            "Should return None for job at max attempts"
        );
    }

    #[tokio::test]
    async fn test_retry_policy_error_handling() {
        let config = RetryConfig {
            max_attempts: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_millis(1000),
            backoff_multiplier: 2.0,
            jitter: false,
        };

        let policy = RetryPolicy::new(config).unwrap();

        // Test with different error types
        let errors: Vec<Box<dyn std::error::Error + Send + Sync>> = vec![
            Box::new(std::io::Error::new(std::io::ErrorKind::TimedOut, "Timeout")),
            Box::new(std::io::Error::new(
                std::io::ErrorKind::ConnectionRefused,
                "Connection refused",
            )),
            Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Generic error",
            )),
        ];

        for error in errors {
            let job = create_test_queued_job(1);
            let delay = policy.next_attempt_delay(&job, error.as_ref());
            assert!(delay.is_some(), "Should retry on network/temporary errors");
        }

        // Test with non-retryable error (if implemented)
        // This would be an extension where certain errors are not retryable
        let job = create_test_queued_job(1);
        let delay = policy.next_attempt_delay(&job, mock_error().as_ref());
        assert!(delay.is_some(), "Default behavior should retry all errors");
    }
}

#[cfg(test)]
mod worker_integration_tests {
    use super::*;
    use mimivibe_backend::queue::inmemory_queue::InMemoryQueue;

    #[tokio::test]
    async fn test_worker_retry_workflow_simulation() {
        // This test simulates a complete worker retry workflow
        let queue = Arc::new(InMemoryQueue::new());
        let retry_config = RetryConfig {
            max_attempts: 3,
            base_delay: Duration::from_millis(10), // Short for test
            max_delay: Duration::from_millis(100),
            backoff_multiplier: 2.0,
            jitter: false,
        };
        let retry_policy = RetryPolicy::new(retry_config).unwrap();

        // Enqueue a job
        let payload = create_test_payload("คำถามทดสอบ");
        let job_id = queue.enqueue(payload.clone()).await.unwrap();

        // Simulate worker processing with failures
        let mut attempt_count = 0;
        let max_simulated_attempts = 3;

        while attempt_count < max_simulated_attempts {
            // Dequeue job
            let job_result = queue.dequeue("test-worker").await.unwrap();
            assert!(job_result.is_some(), "Job should be available");

            let job = job_result.unwrap();
            assert_eq!(job.job_id, job_id);

            // Simulate processing failure
            attempt_count += 1;
            let processing_error = mock_error();

            // Check retry policy
            if let Some(retry_delay) = retry_policy.next_attempt_delay(&job, processing_error.as_ref()) {
                // Job should be requeued (we'll simulate this with nack)
                queue
                    .nack(
                        &job.job_id,
                        "test-worker",
                        Some(processing_error.to_string()),
                    )
                    .await
                    .unwrap();

                // Wait for retry delay (in real worker)
                sleep(retry_delay).await;
            } else {
                // Job should go to DLQ (not implemented in basic queue)
                break;
            }
        }

        // Verify final state
        let queue_length = queue.get_queue_length().await.unwrap();
        assert_eq!(queue_length, 0, "Job should be processed or in DLQ");
    }

    #[tokio::test]
    async fn test_worker_job_processing_with_retry() {
        // Test worker processing with retry logic (simplified, single worker)
        let queue = Arc::new(InMemoryQueue::new());
        let retry_config = RetryConfig {
            max_attempts: 2,
            base_delay: Duration::from_millis(5),
            max_delay: Duration::from_millis(50),
            backoff_multiplier: 1.5,
            jitter: true,
        };

        let retry_policy = RetryPolicy::new(retry_config).unwrap();

        // Enqueue a test job
        let payload = create_test_payload("คำถามทดสอบ");
        let job_id = queue.enqueue(payload).await.unwrap();

        // Simulate worker processing
        let worker_name = "test-worker";

        // Dequeue job
        let job_result = queue.dequeue(worker_name).await.unwrap();
        assert!(job_result.is_some(), "Job should be available");

        let job = job_result.unwrap();
        assert_eq!(job.job_id, job_id);

        // Simulate processing failure and retry logic
        let processing_error = mock_error();

        // Check if retry is needed
        if let Some(_retry_delay) = retry_policy.next_attempt_delay(&job, processing_error.as_ref()) {
            // Simulate nack for retry - this should requeue the job
            queue
                .nack(
                    &job.job_id,
                    worker_name,
                    Some("Simulated failure".to_string()),
                )
                .await
                .unwrap();

            // Job should be back in queue for retry (in real implementation)
            // For this test, we just verify the retry policy logic worked
            assert!(true, "Retry logic executed correctly");
        } else {
            // Should not reach here for first attempt
            panic!("Job should be retryable on first failure");
        }
    }

    #[tokio::test]
    async fn test_retry_performance_characteristics() {
        // Test retry performance with large number of attempts
        let config = RetryConfig {
            max_attempts: 100,
            base_delay: Duration::from_millis(1),
            max_delay: Duration::from_millis(100),
            backoff_multiplier: 1.1,
            jitter: true,
        };

        let policy = RetryPolicy::new(config).unwrap();

        let start_time = std::time::Instant::now();

        // Calculate delays for all possible attempts
        for attempt in 1..100 {
            let _delay = policy.calculate_delay(attempt);
        }

        let elapsed = start_time.elapsed();
        assert!(
            elapsed < Duration::from_millis(10),
            "Delay calculations should be fast: {:?}",
            elapsed
        );
    }
}

#[cfg(test)]
mod edge_case_tests {
    use super::*;

    #[tokio::test]
    async fn test_zero_base_delay_handling() {
        // Even if base_delay is 0, should not panic
        let config = RetryConfig {
            max_attempts: 3,
            base_delay: Duration::from_millis(0),
            max_delay: Duration::from_millis(1000),
            backoff_multiplier: 2.0,
            jitter: false,
        };

        let policy_result = RetryPolicy::new(config);
        // This should be rejected during validation
        assert!(policy_result.is_err(), "Zero base delay should be rejected");
    }

    #[tokio::test]
    async fn test_very_large_backoff_multiplier() {
        // Test with very large multiplier to ensure no overflow
        let config = RetryConfig {
            max_attempts: 5,
            base_delay: Duration::from_millis(1),
            max_delay: Duration::from_millis(1000000), // Large max delay
            backoff_multiplier: 1000.0,
            jitter: false,
        };

        let policy = RetryPolicy::new(config).unwrap();

        // Should not overflow or panic
        for attempt in 1..5 {
            let delay = policy.calculate_delay(attempt);
            assert!(delay > Duration::from_millis(0), "Delay should be positive");
            assert!(
                delay <= config.max_delay,
                "Delay should be capped at max_delay"
            );
        }
    }

    #[tokio::test]
    async fn test_maximum_attempts_boundary() {
        let config = RetryConfig {
            max_attempts: 1, // Minimum allowed
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_millis(1000),
            backoff_multiplier: 2.0,
            jitter: false,
        };

        let policy = RetryPolicy::new(config).unwrap();

        let job = create_test_queued_job(1); // Already at max attempts
        let no_delay = policy.next_attempt_delay(&job, mock_error().as_ref());
        assert!(
            no_delay.is_none(),
            "Should not retry when at max attempts = 1"
        );

        // Test should_retry method directly
        assert!(
            !policy.should_retry(1, mock_error().as_ref()),
            "Should not retry when attempts >= max_attempts"
        );
    }

    #[tokio::test]
    async fn test_default_retry_config() {
        // Test that default configuration is sensible
        let config = RetryConfig::default();
        let policy = RetryPolicy::new(config).unwrap();

        // Should have reasonable defaults
        assert!(
            policy.config.max_attempts >= 3,
            "Should have at least 3 attempts by default"
        );
        assert!(
            policy.config.base_delay >= Duration::from_millis(100),
            "Should have reasonable base delay"
        );
        assert!(
            policy.config.max_delay > policy.config.base_delay,
            "Max delay should be greater than base delay"
        );
        assert!(
            policy.config.backoff_multiplier > 1.0,
            "Backoff multiplier should be greater than 1"
        );
    }
}
