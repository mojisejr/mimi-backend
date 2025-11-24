//! Unit tests for performance metrics module
//!
//! Tests metric collection, reporting, and percentile calculations.

#[cfg(test)]
mod metrics_tests {
    use mimivibe_backend::monitor::metrics::MetricsCollector;

    #[test]
    fn test_metrics_collector_creation() {
        let collector = MetricsCollector::new();
        assert_eq!(collector.get_backlog_size(), 0);
    }

    #[test]
    fn test_record_latency() {
        let mut collector = MetricsCollector::new();
        collector.record_latency(100);
        collector.record_latency(200);
        collector.record_latency(300);

        let metrics = collector.calculate_metrics();
        assert!(metrics.latency.count > 0);
    }

    #[test]
    fn test_percentile_calculation_p95() {
        let mut collector = MetricsCollector::new();

        // Record 100 latency measurements
        for i in 1..=100 {
            collector.record_latency(i);
        }

        let metrics = collector.calculate_metrics();

        // P95 should be around 95
        assert!(metrics.latency.p95 >= 90 && metrics.latency.p95 <= 100);
    }

    #[test]
    fn test_percentile_calculation_p99() {
        let mut collector = MetricsCollector::new();

        // Record 100 latency measurements
        for i in 1..=100 {
            collector.record_latency(i);
        }

        let metrics = collector.calculate_metrics();

        // P99 should be around 99
        assert!(metrics.latency.p99 >= 95 && metrics.latency.p99 <= 100);
    }

    #[test]
    fn test_backlog_tracking() {
        let mut collector = MetricsCollector::new();

        collector.increment_backlog();
        assert_eq!(collector.get_backlog_size(), 1);

        collector.increment_backlog();
        collector.increment_backlog();
        assert_eq!(collector.get_backlog_size(), 3);

        collector.decrement_backlog();
        assert_eq!(collector.get_backlog_size(), 2);
    }

    #[test]
    fn test_metrics_logging() {
        let mut collector = MetricsCollector::new();

        collector.record_latency(50);
        collector.record_latency(100);
        collector.record_latency(150);
        collector.increment_backlog();

        let metrics = collector.calculate_metrics();

        // Test that metrics can be logged without panicking
        metrics.log();
    }

    #[test]
    fn test_latency_metrics_with_single_value() {
        let mut collector = MetricsCollector::new();
        collector.record_latency(100);

        let metrics = collector.calculate_metrics();

        assert_eq!(metrics.latency.count, 1);
        assert_eq!(metrics.latency.p95, 100);
        assert_eq!(metrics.latency.p99, 100);
    }

    #[test]
    fn test_latency_metrics_with_no_data() {
        let collector = MetricsCollector::new();
        let metrics = collector.calculate_metrics();

        assert_eq!(metrics.latency.count, 0);
        assert_eq!(metrics.latency.p95, 0);
        assert_eq!(metrics.latency.p99, 0);
    }

    #[test]
    fn test_performance_metrics_serialization() {
        let mut collector = MetricsCollector::new();
        collector.record_latency(100);
        collector.increment_backlog();

        let metrics = collector.calculate_metrics();

        // Test that metrics can be serialized to JSON
        let json = serde_json::to_string(&metrics);
        assert!(json.is_ok());
    }

    #[test]
    fn test_backlog_cannot_go_negative() {
        let mut collector = MetricsCollector::new();

        // Try to decrement when backlog is 0
        collector.decrement_backlog();

        // Should remain at 0, not go negative
        assert_eq!(collector.get_backlog_size(), 0);
    }

    #[test]
    fn test_metrics_reset() {
        let mut collector = MetricsCollector::new();

        collector.record_latency(100);
        collector.increment_backlog();

        collector.reset();

        assert_eq!(collector.get_backlog_size(), 0);
        let metrics = collector.calculate_metrics();
        assert_eq!(metrics.latency.count, 0);
    }
}
