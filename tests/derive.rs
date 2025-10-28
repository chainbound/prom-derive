use std::time::Duration;

use prometheus::Encoder as _;

/// This is a struct that contains the metrics for the application.
///
/// # Explanation
///
/// - Deriving `PrometheusMetrics` will generate the metrics for the struct.
/// - #[metrics(prefix = "app", static_labels = [("host", "localhost"), ("port", "8080")])]
/// is used to set the prefix and static labels for the metrics.
///
/// - Doc comments on the fields are used to generate the documentation for the metric.
/// - #[metric] attribute defines the metric name, and labels, and potentially other options for that metric type (like buckets)
/// - The type of the field is used to determine the metric type.
/// - Deriving `Default` will generate a default instance of the struct with the metrics initialized and described. Counters and gauges
/// will be initialized to 0.
#[prom_derive::metrics(scope = "app")]
struct AppMetrics {
    /// The total number of HTTP requests.
    #[metric(rename = "http_requests_total", labels = ["method", "path"])]
    http_requests_total: IntCounter,

    /// The duration of HTTP requests.
    #[metric(labels = ["method", "path"], buckets = [0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0], sample = 0.1)]
    http_requests_duration: Histogram,

    /// The current number of active users.
    #[metric(rename = "current_users", labels = ["service"])]
    current_users: IntGauge,
}

#[test]
fn it_works() {
    // Register with default registry, no static labels
    // let app_metrics = AppMetrics::default();

    // OR use a custom registry, static labels with builder-style API
    let registry = prometheus::default_registry();
    let app_metrics = AppMetrics::builder()
        .with_registry(&registry)
        .with_label("host", "localhost") // These define the static labels for these metrics
        .with_label("port", "8080")
        .build(); // Build the metrics instance

    app_metrics
        .http_requests_total()
        .method("GET")
        .path("/")
        .inc();

    // Increment all GET requests by 1
    app_metrics.http_requests_total().method("GET").inc();

    // Increment all POST requests by 2
    app_metrics.http_requests_total().method("POST").inc_by(2);

    // Set the current number of active users for service-1 to 10
    app_metrics.current_users().service("service-1").set(10);
    // Set the current number of active users to 20
    app_metrics.current_users().set(20);

    let duration = Duration::from_secs(1);
    app_metrics
        .http_requests_duration()
        .method("GET")
        .path("/")
        .observe(duration.as_secs_f64());

    let encoder = prometheus::TextEncoder::new();
    let metric_families = registry.gather(); // Wait, need to expose registry

    let mut buffer = vec![];
    encoder.encode(&metric_families, &mut buffer).unwrap();

    let output = String::from_utf8(buffer).unwrap();
    println!("\n=== Prometheus Metrics Output ===\n{}", output);
}
