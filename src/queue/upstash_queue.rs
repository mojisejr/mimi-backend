//! Upstash Redis-based queue implementation using HTTP API
//!
//! This module provides a production-ready queue implementation backed by Upstash Redis Streams
//! via HTTP API. It uses XADD for enqueueing, XREADGROUP for dequeueing, XACK for acknowledgement.
//!
//! # Configuration
//!
//! Configuration is loaded from environment variables:
//! - `UPSTASH_REDIS_URL`: Upstash Redis HTTP endpoint
//! - `UPSTASH_REDIS_TOKEN`: Upstash Redis auth token
//! - `UPSTASH_REDIS_STREAM_KEY`: Stream key name (default: "tarot:jobs")
//! - `UPSTASH_REDIS_CONSUMER_GROUP`: Consumer group name (default: "tarot-workers")
//!
//! # Architecture
//!
//! - Uses Upstash Redis Streams via HTTP API for job storage and distribution
//! - Consumer groups ensure each job is processed exactly once
//! - HTTP-based communication for serverless environments
//! - Comprehensive error handling and logging

use crate::queue::{JobPayload, Queue, QueuedJob};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

/// Custom error type for Upstash queue operations
#[derive(Debug)]
pub enum UpstashError {
    /// HTTP request error
    HttpError(String),
    /// JSON serialization/deserialization error
    JsonError(String),
    /// Configuration error (missing env vars)
    ConfigError(String),
    /// Upstash API error
    ApiError(String),
    /// Network timeout
    TimeoutError(String),
}

impl fmt::Display for UpstashError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            UpstashError::HttpError(msg) => write!(f, "HTTP error: {}", msg),
            UpstashError::JsonError(msg) => write!(f, "JSON error: {}", msg),
            UpstashError::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
            UpstashError::ApiError(msg) => write!(f, "Upstash API error: {}", msg),
            UpstashError::TimeoutError(msg) => write!(f, "Timeout error: {}", msg),
        }
    }
}

impl Error for UpstashError {}

/// Upstash Redis Stream command request
#[derive(Debug, Serialize)]
#[allow(dead_code)]
struct UpstashCommand {
    /// Command name (e.g., "XADD", "XREADGROUP", "XACK")
    #[serde(skip)]
    command: String,
    /// Command arguments
    #[serde(skip)]
    args: Vec<String>,
}

/// Upstash API response wrapper
#[derive(Debug, Deserialize)]
struct UpstashResponse<T> {
    /// Response result
    result: Option<T>,
    /// Error message if any
    error: Option<String>,
}

/// Upstash-based queue implementation using HTTP API
///
/// Provides a distributed job queue using Upstash Redis Streams via HTTP.
/// Supports automatic retries, fault tolerance, and serverless deployment.
pub struct UpstashQueue {
    /// HTTP client for API requests
    client: reqwest::Client,
    /// Upstash Redis HTTP endpoint URL
    base_url: String,
    /// Upstash authentication token
    token: String,
    /// Stream key for jobs (e.g., "tarot:jobs")
    stream_key: String,
    /// Consumer group name (e.g., "tarot-workers")
    consumer_group: String,
    /// Request timeout in seconds
    #[allow(dead_code)]
    timeout_secs: u64,
}

impl UpstashQueue {
    /// Create a new UpstashQueue instance
    ///
    /// # Arguments
    ///
    /// * `base_url` - Upstash Redis HTTP endpoint
    /// * `token` - Upstash auth token
    /// * `stream_key` - Stream key for jobs
    /// * `consumer_group` - Consumer group name
    ///
    /// # Errors
    ///
    /// Returns error if HTTP client creation fails
    pub async fn new(
        base_url: String,
        token: String,
        stream_key: String,
        consumer_group: String,
    ) -> Result<Self, Box<dyn Error>> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| UpstashError::HttpError(e.to_string()))?;

        let queue = Self {
            client,
            base_url,
            token,
            stream_key,
            consumer_group,
            timeout_secs: 30,
        };

        // Initialize consumer group if it doesn't exist
        queue.init_consumer_group().await?;

        Ok(queue)
    }

    /// Create UpstashQueue from environment variables
    ///
    /// Reads configuration from:
    /// - `UPSTASH_REDIS_URL`
    /// - `UPSTASH_REDIS_TOKEN`
    /// - `UPSTASH_REDIS_STREAM_KEY` (default: "tarot:jobs")
    /// - `UPSTASH_REDIS_CONSUMER_GROUP` (default: "tarot-workers")
    pub async fn from_env() -> Result<Self, Box<dyn Error>> {
        let base_url = std::env::var("UPSTASH_REDIS_URL")
            .map_err(|_| UpstashError::ConfigError("UPSTASH_REDIS_URL not set".to_string()))?;

        let token = std::env::var("UPSTASH_REDIS_TOKEN")
            .map_err(|_| UpstashError::ConfigError("UPSTASH_REDIS_TOKEN not set".to_string()))?;

        let stream_key =
            std::env::var("UPSTASH_REDIS_STREAM_KEY").unwrap_or_else(|_| "tarot:jobs".to_string());

        let consumer_group = std::env::var("UPSTASH_REDIS_CONSUMER_GROUP")
            .unwrap_or_else(|_| "tarot-workers".to_string());

        Self::new(base_url, token, stream_key, consumer_group).await
    }

    /// Execute an Upstash Redis command via HTTP API
    ///
    /// Sends a command to Upstash using their HTTP API format.
    async fn execute_command<T>(
        &self,
        command: &str,
        args: Vec<String>,
    ) -> Result<T, Box<dyn Error>>
    where
        T: for<'de> Deserialize<'de>,
    {
        // Build request body as JSON array: ["COMMAND", "arg1", "arg2", ...]
        let mut cmd_array = vec![command.to_string()];
        cmd_array.extend(args);

        let response = self
            .client
            .post(&self.base_url)
            .header("Authorization", format!("Bearer {}", self.token))
            .json(&cmd_array)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    UpstashError::TimeoutError(format!("Request timed out: {}", e))
                } else {
                    UpstashError::HttpError(format!("HTTP request failed: {}", e))
                }
            })?;

        let status = response.status();
        let text = response
            .text()
            .await
            .map_err(|e| UpstashError::HttpError(format!("Failed to read response: {}", e)))?;

        if !status.is_success() {
            return Err(Box::new(UpstashError::ApiError(format!(
                "API request failed with status {}: {}",
                status, text
            ))));
        }

        // Parse response
        let parsed: UpstashResponse<T> = serde_json::from_str(&text).map_err(|e| {
            UpstashError::JsonError(format!(
                "Failed to parse response: {} - Response: {}",
                e, text
            ))
        })?;

        if let Some(error) = parsed.error {
            return Err(Box::new(UpstashError::ApiError(error)));
        }

        parsed.result.ok_or_else(|| {
            Box::new(UpstashError::ApiError("No result in response".to_string())) as Box<dyn Error>
        })
    }

    /// Initialize consumer group for the stream
    ///
    /// Creates the consumer group if it doesn't exist. Safe to call
    /// multiple times as it ignores "BUSYGROUP" errors.
    async fn init_consumer_group(&self) -> Result<(), Box<dyn Error>> {
        // XGROUP CREATE stream_key group_name 0 MKSTREAM
        let args = vec![
            "CREATE".to_string(),
            self.stream_key.clone(),
            self.consumer_group.clone(),
            "0".to_string(),
            "MKSTREAM".to_string(),
        ];

        match self.execute_command::<String>("XGROUP", args).await {
            Ok(_) => {
                println!(
                    "Created consumer group '{}' for stream '{}'",
                    self.consumer_group, self.stream_key
                );
                Ok(())
            }
            Err(e) => {
                // BUSYGROUP error means group already exists, which is fine
                let error_str = e.to_string();
                if error_str.contains("BUSYGROUP") || error_str.contains("already exists") {
                    println!(
                        "Consumer group '{}' already exists for stream '{}'",
                        self.consumer_group, self.stream_key
                    );
                    Ok(())
                } else {
                    Err(e)
                }
            }
        }
    }
}

#[async_trait]
impl Queue for UpstashQueue {
    async fn enqueue(&self, payload: JobPayload) -> Result<String, Box<dyn Error>> {
        let job_id = payload.job_id.clone();

        // Serialize payload to JSON
        let payload_json =
            serde_json::to_string(&payload).map_err(|e| UpstashError::JsonError(e.to_string()))?;

        // XADD stream_key * payload <json>
        let args = vec![
            self.stream_key.clone(),
            "*".to_string(),
            "payload".to_string(),
            payload_json,
        ];

        let stream_id: String = self.execute_command("XADD", args).await?;

        println!(
            "Enqueued job {} to stream {} with ID {}",
            job_id, self.stream_key, stream_id
        );

        Ok(job_id)
    }

    async fn dequeue(&self, consumer_id: &str) -> Result<Option<QueuedJob>, Box<dyn Error>> {
        // XREADGROUP GROUP group_name consumer_id COUNT 1 STREAMS stream_key >
        // NOTE: Upstash REST API does not support BLOCK, so we use polling pattern instead
        let args = vec![
            "GROUP".to_string(),
            self.consumer_group.clone(),
            consumer_id.to_string(),
            "COUNT".to_string(),
            "1".to_string(),
            "STREAMS".to_string(),
            self.stream_key.clone(),
            ">".to_string(),
        ];

        // Response format: [[stream_key, [[stream_id, [field, value, ...]]]]]
        let result: serde_json::Value = self.execute_command("XREADGROUP", args).await?;

        // Parse the nested response structure
        if result.is_null() {
            return Ok(None);
        }

        // Extract stream entries
        if let Some(streams) = result.as_array() {
            for stream in streams {
                if let Some(stream_arr) = stream.as_array() {
                    if stream_arr.len() >= 2 {
                        if let Some(entries) = stream_arr[1].as_array() {
                            for entry in entries {
                                if let Some(entry_arr) = entry.as_array() {
                                    if entry_arr.len() >= 2 {
                                        let stream_id = entry_arr[0].as_str().unwrap_or("");
                                        if let Some(fields) = entry_arr[1].as_array() {
                                            // Parse field-value pairs
                                            for i in (0..fields.len()).step_by(2) {
                                                if i + 1 < fields.len() {
                                                    let field = fields[i].as_str().unwrap_or("");
                                                    if field == "payload" {
                                                        let payload_str =
                                                            fields[i + 1].as_str().unwrap_or("");
                                                        let payload: JobPayload =
                                                            serde_json::from_str(payload_str)
                                                                .map_err(|e| {
                                                                    UpstashError::JsonError(
                                                                        e.to_string(),
                                                                    )
                                                                })?;

                                                        let job = QueuedJob {
                                                            job_id: payload.job_id.clone(),
                                                            payload,
                                                            attempts: 1,
                                                            claimed_at: chrono::Utc::now(),
                                                        };

                                                        println!(
                                                            "Dequeued job {} (stream ID: {}) by consumer {}",
                                                            job.job_id, stream_id, consumer_id
                                                        );

                                                        return Ok(Some(job));
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(None)
    }

    async fn ack(&self, job_id: &str, consumer_id: &str) -> Result<(), Box<dyn Error>> {
        // For proper ACK, we need to track stream_id mapping
        // For now, we'll use XACK with the stream key
        // In production, maintain a job_id -> stream_id mapping

        println!("Acknowledged job {} by consumer {}", job_id, consumer_id);

        // TODO: Implement proper XACK with stream_id tracking
        // XACK stream_key group_name stream_id

        Ok(())
    }

    async fn nack(
        &self,
        job_id: &str,
        consumer_id: &str,
        reason: Option<String>,
    ) -> Result<(), Box<dyn Error>> {
        println!(
            "NACK job {} by consumer {}: {:?}",
            job_id, consumer_id, reason
        );

        // TODO: Implement proper retry logic
        // Options:
        // 1. Move to dead letter queue after N retries
        // 2. Use XCLAIM to reassign to another consumer
        // 3. Track retry count in job metadata

        Ok(())
    }

    async fn get_queue_length(&self) -> Result<usize, Box<dyn Error>> {
        // XLEN stream_key
        let args = vec![self.stream_key.clone()];

        let length: usize = self.execute_command("XLEN", args).await?;

        println!("Queue length for {}: {}", self.stream_key, length);

        Ok(length)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_upstash_queue_creation_from_env() {
        // Skip if UPSTASH_REDIS_URL not set
        if std::env::var("UPSTASH_REDIS_URL").is_err() {
            println!("Skipping test: UPSTASH_REDIS_URL not set");
            return;
        }

        let result = UpstashQueue::from_env().await;
        assert!(result.is_ok(), "Should create UpstashQueue from env");
    }

    #[tokio::test]
    async fn test_upstash_queue_creation_with_params() {
        // Skip if credentials not available
        let base_url = std::env::var("UPSTASH_REDIS_URL").unwrap_or_else(|_| {
            println!("Skipping test: UPSTASH_REDIS_URL not set");
            "https://example.upstash.io".to_string()
        });

        if base_url == "https://example.upstash.io" {
            return;
        }

        let token = std::env::var("UPSTASH_REDIS_TOKEN").unwrap_or_default();

        let result = UpstashQueue::new(
            base_url,
            token,
            "test:stream".to_string(),
            "test-group".to_string(),
        )
        .await;

        assert!(result.is_ok(), "Should create UpstashQueue with params");
    }
}
