//! Monitor module for performance metrics and health checks
//!
//! This module provides functionality for tracking performance metrics
//! (p95, p99, backlog, latency) and health endpoint implementation.

pub mod health;
pub mod metrics;

pub use health::{HealthCheck, HealthStatus};
pub use metrics::{LatencyMetrics, MetricsCollector, PerformanceMetrics};
