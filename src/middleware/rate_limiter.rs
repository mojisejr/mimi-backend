//! Rate Limiting Middleware
//!
//! Redis-based rate limiting with configurable cooldown periods.
//! Prevents spam requests by enforcing time limits between requests.

use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use redis::AsyncCommands;
use serde_json::json;

/// Rate limiter configuration
#[derive(Debug, Clone)]
pub struct RateLimiterConfig {
    /// Redis key prefix
    pub key_prefix: String,

    /// Time window for rate limiting (in seconds)
    pub window_seconds: u64,

    /// Maximum requests allowed within the time window
    pub max_requests: u32,
}

impl Default for RateLimiterConfig {
    fn default() -> Self {
        Self {
            key_prefix: "rate_limit".to_string(),
            window_seconds: 600, // 10 minutes
            max_requests: 1,     // Only 1 request per 10 minutes
        }
    }
}

/// Rate limiter state
#[derive(Debug, Clone)]
pub struct RateLimiterState {
    pub redis_client: redis::Client,
    pub config: RateLimiterConfig,
}

impl RateLimiterState {
    /// Create a new rate limiter state
    pub fn new(redis_client: redis::Client, config: RateLimiterConfig) -> Self {
        Self {
            redis_client,
            config,
        }
    }

    /// Create with default configuration
    pub fn default_config(redis_client: redis::Client) -> Self {
        Self::new(redis_client, RateLimiterConfig::default())
    }
}

/// Extract client IP address from request headers
pub fn extract_client_ip(headers: &HeaderMap) -> Option<String> {
    // Try various common headers for client IP
    let ip_headers = [
        "x-forwarded-for",
        "x-real-ip",
        "cf-connecting-ip",
        "x-client-ip",
        "x-forwarded",
        "forwarded-for",
        "forwarded",
    ];

    for header_name in &ip_headers {
        if let Some(header_value) = headers.get(*header_name) {
            if let Ok(ip_str) = header_value.to_str() {
                // X-Forwarded-For can contain multiple IPs, take the first one
                let ip = ip_str.split(',').next().unwrap_or("").trim();
                if !ip.is_empty() {
                    return Some(ip.to_string());
                }
            }
        }
    }

    None
}

/// Rate limiting middleware function
pub async fn rate_limit_middleware(
    State(state): State<RateLimiterState>,
    request: Request,
    next: Next,
) -> Result<Response, Response> {
    let headers = request.headers().clone();
    let rate_limit_key = extract_rate_limit_key(&headers, &state.config.key_prefix);

    match check_rate_limit(&state.redis_client, &rate_limit_key, &state.config).await {
        Ok(allowed) => {
            if allowed {
                // Rate limit check passed, continue with request
                Ok(next.run(request).await)
            } else {
                // Rate limit exceeded
                let error_response = json!({
                    "error": "Rate limit exceeded. Please wait 10 minutes before next request.",
                    "code": "RATE_LIMIT_EXCEEDED",
                    "retry_after": state.config.window_seconds
                });

                let response = axum::Json(error_response).into_response();
                let mut error_response = response.into_response();

                // Add Retry-After header
                if let Some(retry_after) = error_response.headers_mut().get_mut("Retry-After") {
                    *retry_after = state.config.window_seconds.to_string().parse().unwrap();
                } else {
                    error_response.headers_mut().insert(
                        "Retry-After",
                        state.config.window_seconds.to_string().parse().unwrap(),
                    );
                }

                *error_response.status_mut() = StatusCode::TOO_MANY_REQUESTS;

                Err(error_response)
            }
        }
        Err(e) => {
            // Redis error - allow request to proceed (fail open)
            eprintln!(
                "Rate limiter Redis error: {}. Allowing request to proceed.",
                e
            );
            Ok(next.run(request).await)
        }
    }
}

/// Extract rate limit key from request
fn extract_rate_limit_key(headers: &HeaderMap, key_prefix: &str) -> String {
    // Try to get user ID from custom header (if authenticated)
    if let Some(user_id_header) = headers.get("x-user-id") {
        if let Ok(user_id) = user_id_header.to_str() {
            return format!("{}:user:{}", key_prefix, user_id);
        }
    }

    // Fallback to client IP
    if let Some(client_ip) = extract_client_ip(headers) {
        return format!("{}:ip:{}", key_prefix, client_ip);
    }

    // Final fallback to anonymous
    format!("{}:anonymous", key_prefix)
}

/// Check if request is allowed based on rate limit
async fn check_rate_limit(
    redis_client: &redis::Client,
    key: &str,
    config: &RateLimiterConfig,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let mut conn = redis_client.get_multiplexed_async_connection().await?;

    // Use Redis INCR with EXPIRE for simple rate limiting
    let current_count: i64 = conn.incr(key, 1).await?;

    // If this is the first request, set expiration
    if current_count == 1 {
        let _: () = conn.expire(key, config.window_seconds as i64).await?;
    }

    // Check if limit exceeded
    Ok(current_count <= config.max_requests as i64)
}

/// Get current rate limit status for a key
pub async fn get_rate_limit_status(
    redis_client: &redis::Client,
    key: &str,
) -> Result<RateLimitStatus, Box<dyn std::error::Error + Send + Sync>> {
    let mut conn = redis_client.get_multiplexed_async_connection().await?;

    let current_count: Option<i64> = conn.get(key).await?;
    let ttl: i64 = conn.ttl(key).await?;

    Ok(RateLimitStatus {
        current_requests: current_count.unwrap_or(0) as u32,
        max_requests: 1, // This should match our config
        reset_time_seconds: if ttl > 0 { Some(ttl as u64) } else { None },
    })
}

/// Rate limit status information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RateLimitStatus {
    pub current_requests: u32,
    pub max_requests: u32,
    pub reset_time_seconds: Option<u64>,
}

impl RateLimitStatus {
    /// Check if rate limit is exceeded
    pub fn is_exceeded(&self) -> bool {
        self.current_requests > self.max_requests
    }

    /// Get remaining requests
    pub fn remaining_requests(&self) -> u32 {
        if self.current_requests >= self.max_requests {
            0
        } else {
            self.max_requests - self.current_requests
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_rate_limiter_config() {
        let config = RateLimiterConfig::default();
        assert_eq!(config.window_seconds, 600);
        assert_eq!(config.max_requests, 1);
        assert_eq!(config.key_prefix, "rate_limit");
    }

    #[test]
    fn test_rate_limit_status_creation() {
        let status = RateLimitStatus {
            current_requests: 1,
            max_requests: 1,
            reset_time_seconds: Some(300),
        };

        assert!(status.is_exceeded());
        assert_eq!(status.remaining_requests(), 0);
    }

    #[test]
    fn test_rate_limit_status_not_exceeded() {
        let status = RateLimitStatus {
            current_requests: 0,
            max_requests: 1,
            reset_time_seconds: Some(300),
        };

        assert!(!status.is_exceeded());
        assert_eq!(status.remaining_requests(), 1);
    }

    #[test]
    fn test_extract_client_ip_single_ip() {
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", "192.168.1.1".parse().unwrap());

        let ip = extract_client_ip(&headers);
        assert_eq!(ip, Some("192.168.1.1".to_string()));
    }

    #[test]
    fn test_extract_client_ip_multiple_ips() {
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", "192.168.1.1, 10.0.0.1".parse().unwrap());

        let ip = extract_client_ip(&headers);
        assert_eq!(ip, Some("192.168.1.1".to_string()));
    }

    #[test]
    fn test_extract_client_ip_no_headers() {
        let headers = HeaderMap::new();

        let ip = extract_client_ip(&headers);
        assert_eq!(ip, None);
    }

    #[test]
    fn test_extract_rate_limit_key_with_user_id() {
        let mut headers = HeaderMap::new();
        headers.insert("x-user-id", "user123".parse().unwrap());

        let key = extract_rate_limit_key(&headers, "rate_limit");
        assert_eq!(key, "rate_limit:user:user123");
    }

    #[test]
    fn test_extract_rate_limit_key_with_ip() {
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", "192.168.1.1".parse().unwrap());

        let key = extract_rate_limit_key(&headers, "rate_limit");
        assert_eq!(key, "rate_limit:ip:192.168.1.1");
    }

    #[test]
    fn test_extract_rate_limit_key_anonymous() {
        let headers = HeaderMap::new();

        let key = extract_rate_limit_key(&headers, "rate_limit");
        assert_eq!(key, "rate_limit:anonymous");
    }
}
