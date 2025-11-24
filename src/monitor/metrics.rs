//! Performance metrics module
//!
//! Provides functionality for tracking and calculating performance metrics
//! including p95, p99 percentiles, backlog size, and latency measurements.

use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

/// Latency metrics with percentile calculations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyMetrics {
    /// Number of latency measurements
    pub count: usize,
    /// 95th percentile latency in milliseconds
    pub p95: u64,
    /// 99th percentile latency in milliseconds
    pub p99: u64,
    /// Average latency in milliseconds
    pub avg: u64,
    /// Minimum latency in milliseconds
    pub min: u64,
    /// Maximum latency in milliseconds
    pub max: u64,
}

impl LatencyMetrics {
    /// Create new empty latency metrics
    pub fn new() -> Self {
        Self {
            count: 0,
            p95: 0,
            p99: 0,
            avg: 0,
            min: 0,
            max: 0,
        }
    }

    /// Calculate percentile from sorted latency values
    fn calculate_percentile(sorted_values: &[u64], percentile: f64) -> u64 {
        if sorted_values.is_empty() {
            return 0;
        }

        let index = ((sorted_values.len() as f64 * percentile).ceil() as usize).saturating_sub(1);
        sorted_values.get(index).copied().unwrap_or(0)
    }

    /// Create latency metrics from raw measurements
    pub fn from_measurements(measurements: &[u64]) -> Self {
        if measurements.is_empty() {
            return Self::new();
        }

        let mut sorted = measurements.to_vec();
        sorted.sort_unstable();

        let count = sorted.len();
        let p95 = Self::calculate_percentile(&sorted, 0.95);
        let p99 = Self::calculate_percentile(&sorted, 0.99);
        let sum: u64 = sorted.iter().sum();
        let avg = sum / count as u64;
        let min = *sorted.first().unwrap_or(&0);
        let max = *sorted.last().unwrap_or(&0);

        Self {
            count,
            p95,
            p99,
            avg,
            min,
            max,
        }
    }
}

impl Default for LatencyMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Performance metrics container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Latency metrics
    pub latency: LatencyMetrics,
    /// Current backlog size
    pub backlog_size: usize,
    /// Timestamp when metrics were calculated
    pub timestamp: String,
}

impl PerformanceMetrics {
    /// Create new performance metrics
    pub fn new(latency: LatencyMetrics, backlog_size: usize) -> Self {
        use chrono::Utc;
        Self {
            latency,
            backlog_size,
            timestamp: Utc::now().to_rfc3339(),
        }
    }

    /// Log metrics to stdout
    pub fn log(&self) {
        println!("=== Performance Metrics ===");
        println!("Timestamp: {}", self.timestamp);
        println!("Backlog Size: {}", self.backlog_size);
        println!("Latency Stats:");
        println!("  Count: {}", self.latency.count);
        println!("  P95: {} ms", self.latency.p95);
        println!("  P99: {} ms", self.latency.p99);
        println!("  Avg: {} ms", self.latency.avg);
        println!("  Min: {} ms", self.latency.min);
        println!("  Max: {} ms", self.latency.max);
        println!("==========================");
    }
}

/// Metrics collector for tracking performance data
pub struct MetricsCollector {
    latency_measurements: Arc<Mutex<Vec<u64>>>,
    backlog_size: Arc<Mutex<usize>>,
}

impl MetricsCollector {
    /// Create new metrics collector
    pub fn new() -> Self {
        Self {
            latency_measurements: Arc::new(Mutex::new(Vec::new())),
            backlog_size: Arc::new(Mutex::new(0)),
        }
    }

    /// Record a latency measurement in milliseconds
    pub fn record_latency(&mut self, latency_ms: u64) {
        if let Ok(mut measurements) = self.latency_measurements.lock() {
            measurements.push(latency_ms);
        }
    }

    /// Increment backlog counter
    pub fn increment_backlog(&mut self) {
        if let Ok(mut backlog) = self.backlog_size.lock() {
            *backlog += 1;
        }
    }

    /// Decrement backlog counter (cannot go below 0)
    pub fn decrement_backlog(&mut self) {
        if let Ok(mut backlog) = self.backlog_size.lock() {
            if *backlog > 0 {
                *backlog -= 1;
            }
        }
    }

    /// Get current backlog size
    pub fn get_backlog_size(&self) -> usize {
        *self.backlog_size.lock().unwrap_or_else(|_| {
            // In case of poisoned lock, return 0
            panic!("Failed to acquire backlog lock");
        })
    }

    /// Calculate performance metrics from collected data
    pub fn calculate_metrics(&self) -> PerformanceMetrics {
        let measurements = self.latency_measurements.lock().unwrap();
        let latency = LatencyMetrics::from_measurements(&measurements);
        let backlog_size = self.get_backlog_size();

        PerformanceMetrics::new(latency, backlog_size)
    }

    /// Reset all metrics
    pub fn reset(&mut self) {
        if let Ok(mut measurements) = self.latency_measurements.lock() {
            measurements.clear();
        }
        if let Ok(mut backlog) = self.backlog_size.lock() {
            *backlog = 0;
        }
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_latency_metrics_from_empty_measurements() {
        let measurements: Vec<u64> = vec![];
        let metrics = LatencyMetrics::from_measurements(&measurements);

        assert_eq!(metrics.count, 0);
        assert_eq!(metrics.p95, 0);
        assert_eq!(metrics.p99, 0);
    }

    #[test]
    fn test_latency_metrics_from_single_measurement() {
        let measurements = vec![100];
        let metrics = LatencyMetrics::from_measurements(&measurements);

        assert_eq!(metrics.count, 1);
        assert_eq!(metrics.p95, 100);
        assert_eq!(metrics.p99, 100);
        assert_eq!(metrics.avg, 100);
    }

    #[test]
    fn test_percentile_calculation() {
        let measurements: Vec<u64> = (1..=100).collect();
        let metrics = LatencyMetrics::from_measurements(&measurements);

        // P95 should be around 95
        assert!(metrics.p95 >= 90 && metrics.p95 <= 100);
        // P99 should be around 99
        assert!(metrics.p99 >= 95 && metrics.p99 <= 100);
    }

    #[test]
    fn test_metrics_collector_basic_operations() {
        let mut collector = MetricsCollector::new();

        collector.record_latency(100);
        collector.increment_backlog();

        assert_eq!(collector.get_backlog_size(), 1);

        let metrics = collector.calculate_metrics();
        assert_eq!(metrics.latency.count, 1);
        assert_eq!(metrics.backlog_size, 1);
    }
}
