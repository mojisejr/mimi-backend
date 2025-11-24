//! Health check module
//!
//! Provides health check functionality and HTTP endpoint support
//! for monitoring system health status.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Health status enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    /// System is healthy and operational
    Healthy,
    /// System is unhealthy or degraded
    Unhealthy,
}

impl HealthStatus {
    /// Get HTTP status code for health status
    pub fn http_status_code(&self) -> u16 {
        match self {
            HealthStatus::Healthy => 200,
            HealthStatus::Unhealthy => 503,
        }
    }
}

impl fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HealthStatus::Healthy => write!(f, "healthy"),
            HealthStatus::Unhealthy => write!(f, "unhealthy"),
        }
    }
}

/// Component health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    /// Component name
    pub name: String,
    /// Whether component is healthy
    pub healthy: bool,
    /// Optional status message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    /// Overall health status
    pub status: HealthStatus,
    /// Timestamp of health check
    pub timestamp: String,
    /// Individual component health statuses
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub components: HashMap<String, ComponentHealth>,
}

impl HealthCheck {
    /// Create new health check with healthy status
    pub fn new() -> Self {
        use chrono::Utc;
        Self {
            status: HealthStatus::Healthy,
            timestamp: Utc::now().to_rfc3339(),
            components: HashMap::new(),
        }
    }

    /// Get current health status
    pub fn status(&self) -> HealthStatus {
        self.status
    }

    /// Set health status
    pub fn set_status(&mut self, status: HealthStatus) {
        self.status = status;
    }

    /// Add component health check
    pub fn add_component_check(&mut self, name: &str, healthy: bool) {
        self.add_component_check_with_message(name, healthy, None);
    }

    /// Add component health check with message
    pub fn add_component_check_with_message(
        &mut self,
        name: &str,
        healthy: bool,
        message: Option<String>,
    ) {
        let component = ComponentHealth {
            name: name.to_string(),
            healthy,
            message,
        };

        self.components.insert(name.to_string(), component);

        // Update overall status based on component health
        if !healthy {
            self.status = HealthStatus::Unhealthy;
        }
    }

    /// Check if all components are healthy
    pub fn all_components_healthy(&self) -> bool {
        if self.components.is_empty() {
            return true;
        }
        self.components.values().all(|c| c.healthy)
    }

    /// Update overall status based on component checks
    pub fn update_status(&mut self) {
        if self.all_components_healthy() {
            self.status = HealthStatus::Healthy;
        } else {
            self.status = HealthStatus::Unhealthy;
        }
    }

    /// Convert to JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Convert to pretty JSON string
    pub fn to_json_pretty(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

impl Default for HealthCheck {
    fn default() -> Self {
        Self::new()
    }
}

/// Health check handler function for HTTP endpoints
///
/// This function can be used in Axum handlers to provide health endpoint
/// functionality. Returns a tuple of (status_code, json_body).
pub async fn health_check_handler() -> (u16, String) {
    let health = HealthCheck::new();
    let status_code = health.status().http_status_code();
    let body = health.to_json().unwrap_or_else(|_| {
        r#"{"status":"unhealthy","error":"failed to serialize health check"}"#.to_string()
    });

    (status_code, body)
}

/// Health check handler with custom checks
///
/// Performs health checks on specified components and returns appropriate status.
pub async fn health_check_handler_with_checks(
    check_database: bool,
    check_queue: bool,
) -> (u16, String) {
    let mut health = HealthCheck::new();

    // Simulate component checks (in real implementation, these would be actual checks)
    if check_database {
        health.add_component_check("database", true);
    }

    if check_queue {
        health.add_component_check("queue", true);
    }

    health.update_status();

    let status_code = health.status().http_status_code();
    let body = health.to_json().unwrap_or_else(|_| {
        r#"{"status":"unhealthy","error":"failed to serialize health check"}"#.to_string()
    });

    (status_code, body)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_status_display() {
        assert_eq!(HealthStatus::Healthy.to_string(), "healthy");
        assert_eq!(HealthStatus::Unhealthy.to_string(), "unhealthy");
    }

    #[test]
    fn test_health_status_http_codes() {
        assert_eq!(HealthStatus::Healthy.http_status_code(), 200);
        assert_eq!(HealthStatus::Unhealthy.http_status_code(), 503);
    }

    #[test]
    fn test_health_check_default_is_healthy() {
        let health = HealthCheck::new();
        assert_eq!(health.status(), HealthStatus::Healthy);
    }

    #[test]
    fn test_health_check_json_serialization() {
        let health = HealthCheck::new();
        let json = health.to_json();
        assert!(json.is_ok());
    }

    #[test]
    fn test_component_health_check() {
        let mut health = HealthCheck::new();
        health.add_component_check("database", true);

        assert_eq!(health.components.len(), 1);
        assert_eq!(health.status(), HealthStatus::Healthy);
    }

    #[test]
    fn test_unhealthy_component_updates_status() {
        let mut health = HealthCheck::new();
        health.add_component_check("database", false);

        assert_eq!(health.status(), HealthStatus::Unhealthy);
    }

    #[tokio::test]
    async fn test_health_check_handler() {
        let (status, body) = health_check_handler().await;

        assert_eq!(status, 200);
        assert!(body.contains("status"));
    }
}
