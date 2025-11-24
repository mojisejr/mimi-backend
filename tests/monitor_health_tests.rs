//! Integration tests for health endpoint
//!
//! Tests health check functionality and HTTP endpoint behavior.

#[cfg(test)]
mod health_tests {
    use mimivibe_backend::monitor::health::{HealthCheck, HealthStatus};

    #[test]
    fn test_health_status_healthy() {
        let status = HealthStatus::Healthy;
        assert_eq!(status.to_string(), "healthy");
        assert_eq!(status.http_status_code(), 200);
    }

    #[test]
    fn test_health_status_unhealthy() {
        let status = HealthStatus::Unhealthy;
        assert_eq!(status.to_string(), "unhealthy");
        assert_eq!(status.http_status_code(), 503);
    }

    #[test]
    fn test_health_check_creation() {
        let health_check = HealthCheck::new();
        assert_eq!(health_check.status(), HealthStatus::Healthy);
    }

    #[test]
    fn test_health_check_serialization() {
        let health_check = HealthCheck::new();

        // Test that health check can be serialized to JSON
        let json = serde_json::to_string(&health_check);
        assert!(json.is_ok());

        let json_value = json.unwrap();
        assert!(json_value.contains("status"));
    }

    #[test]
    fn test_health_check_with_unhealthy_status() {
        let mut health_check = HealthCheck::new();
        health_check.set_status(HealthStatus::Unhealthy);

        assert_eq!(health_check.status(), HealthStatus::Unhealthy);
        assert_eq!(health_check.status().http_status_code(), 503);
    }

    #[test]
    fn test_health_check_response_format() {
        let health_check = HealthCheck::new();
        let json = serde_json::to_string(&health_check).unwrap();

        // Verify JSON structure
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed.get("status").is_some());
        assert!(parsed.get("timestamp").is_some());
    }

    #[test]
    fn test_health_check_component_checks() {
        let mut health_check = HealthCheck::new();

        // Test adding component checks
        health_check.add_component_check("database", true);
        health_check.add_component_check("redis", true);

        assert_eq!(health_check.status(), HealthStatus::Healthy);
    }

    #[test]
    fn test_health_check_fails_on_unhealthy_component() {
        let mut health_check = HealthCheck::new();

        // Test that one unhealthy component makes overall status unhealthy
        health_check.add_component_check("database", true);
        health_check.add_component_check("redis", false);

        assert_eq!(health_check.status(), HealthStatus::Unhealthy);
    }

    #[test]
    fn test_health_check_response_includes_components() {
        let mut health_check = HealthCheck::new();
        health_check.add_component_check("database", true);
        health_check.add_component_check("queue", true);

        let json = serde_json::to_string(&health_check).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Verify components are included
        assert!(parsed.get("components").is_some());
    }

    #[test]
    fn test_health_status_equality() {
        assert_eq!(HealthStatus::Healthy, HealthStatus::Healthy);
        assert_eq!(HealthStatus::Unhealthy, HealthStatus::Unhealthy);
        assert_ne!(HealthStatus::Healthy, HealthStatus::Unhealthy);
    }
}
