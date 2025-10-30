# `prometric`

A library for ergonomically generating and using embedded Prometheus metrics in Rust.

Inspired by [metrics-derive](https://github.com/ithacaxyz/metrics-derive), but works directly with [prometheus](https://docs.rs/prometheus/latest/prometheus)
instead of [metrics](https://docs.rs/metrics/latest/metrics), and supports dynamic labels.

## Usage

```rust
use prometric::{Counter, Gauge, Histogram};
use prometric_derive::metrics;

#[metrics(scope = "app")]
struct AppMetrics {
    /// The total number of HTTP requests.
    #[metric(rename = "http_requests_total", labels = ["method", "path"])]
    http_requests: Counter,

    /// The duration of HTTP requests.
    #[metric(labels = ["method", "path"])]
    http_requests_duration: Histogram,

    /// This doc comment will be overwritten by the `help` attribute.
    #[metric(rename = "current_active_users", labels = ["service"], help = "The current number of active users.")]
    current_users: Gauge,

    /// The total number of errors.
    #[metric]
    errors: Counter,
}

fn main() {
    // Register with default registry, no static labels
    let app_metrics = AppMetrics::default();

    // OR use a custom registry, static labels with builder-style API
    let registry = prometheus::default_registry();
    let app_metrics = AppMetrics::builder()
        .with_registry(&registry)
        .with_label("host", "localhost") // These define the static labels for these metrics
        .with_label("port", "8080")
        .build(); // Build the metrics instance

    // No labels
    app_metrics.errors().inc();
    // Labels are converted into method arguments. The method is documented as follows:
    // 
    // ```
    // fn http_requests(&self, method: impl Into<String>, path: impl Into<String>) -> HttpRequestsAccessor
    // 
    // The total number of HTTP requests.
    // * Metric type: Counter
    // * Labels: method, path
    // ```
    app_metrics.http_requests("GET", "/").inc();

    app_metrics.http_requests("GET", "/").inc();

    app_metrics.http_requests("POST", "/").inc_by(2);

    app_metrics.current_users("service-1").set(10);

    app_metrics.current_users("service-1").set(20);

    let duration = Duration::from_secs(1);
    app_metrics
        .http_requests_duration("GET", "/")
        .observe(duration.as_secs_f64());
}
```

Output:
```
# HELP app_current_active_users The current number of active users.
# TYPE app_current_active_users gauge
app_current_active_users{host="localhost",port="8080",service="service-1"} 20
# HELP app_errors The total number of errors.
# TYPE app_errors counter
app_errors{host="localhost",port="8080"} 1
# HELP app_http_requests_duration The duration of HTTP requests.
# TYPE app_http_requests_duration histogram
app_http_requests_duration_bucket{host="localhost",method="GET",path="/",port="8080",le="0.005"} 0
app_http_requests_duration_bucket{host="localhost",method="GET",path="/",port="8080",le="0.01"} 0
app_http_requests_duration_bucket{host="localhost",method="GET",path="/",port="8080",le="0.025"} 0
app_http_requests_duration_bucket{host="localhost",method="GET",path="/",port="8080",le="0.05"} 0
app_http_requests_duration_bucket{host="localhost",method="GET",path="/",port="8080",le="0.1"} 0
app_http_requests_duration_bucket{host="localhost",method="GET",path="/",port="8080",le="0.25"} 0
app_http_requests_duration_bucket{host="localhost",method="GET",path="/",port="8080",le="0.5"} 0
app_http_requests_duration_bucket{host="localhost",method="GET",path="/",port="8080",le="1"} 1
app_http_requests_duration_bucket{host="localhost",method="GET",path="/",port="8080",le="2.5"} 1
app_http_requests_duration_bucket{host="localhost",method="GET",path="/",port="8080",le="5"} 1
app_http_requests_duration_bucket{host="localhost",method="GET",path="/",port="8080",le="10"} 1
app_http_requests_duration_bucket{host="localhost",method="GET",path="/",port="8080",le="+Inf"} 1
app_http_requests_duration_sum{host="localhost",method="GET",path="/",port="8080"} 1
app_http_requests_duration_count{host="localhost",method="GET",path="/",port="8080"} 1
# HELP app_http_requests_total The total number of HTTP requests.
# TYPE app_http_requests_total counter
app_http_requests_total{host="localhost",method="GET",path="/",port="8080"} 2
app_http_requests_total{host="localhost",method="POST",path="/",port="8080"} 2
```