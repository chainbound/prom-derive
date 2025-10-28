# `prom-derive`

> An attribute macro for generating and using embedded Prometheus metrics with an ergonomic API.

## Usage

```rust
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
}
```

Output:
```
# HELP app_current_users The current number of active users.
# TYPE app_current_users gauge
app_current_users{host="localhost",port="8080",service=""} 20
app_current_users{host="localhost",port="8080",service="service-1"} 10
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
app_http_requests_total{host="localhost",method="GET",path="",port="8080"} 1
app_http_requests_total{host="localhost",method="GET",path="/",port="8080"} 1
app_http_requests_total{host="localhost",method="POST",path="",port="8080"} 2
```