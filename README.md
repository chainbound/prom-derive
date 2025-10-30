# `prometric`

A library for ergonomically generating and using embedded Prometheus metrics in Rust.

Inspired by [metrics-derive](https://github.com/ithacaxyz/metrics-derive), but works directly with [prometheus](https://docs.rs/prometheus/latest/prometheus)
instead of [metrics](https://docs.rs/metrics/latest/metrics), and supports dynamic labels.

## Usage
```rust
use prometric_derive::metrics;
use prometric::{Counter, Gauge, Histogram};

// The `scope` attribute is used to set the prefix for the metric names in this struct.
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

// Build the metrics struct with static labels, which will initialize and register the metrics with the default registry.
// A custom registry can be used by passing it to the builder using `with_registry`.
let metrics = AppMetrics::builder().with_label("host", "localhost").with_label("port", "8080").build();

// Metric fields each get an accessor method generated, which can be used to interact with the metric.
// The arguments to the accessor method are the labels for the metric.
metrics.http_requests("GET", "/").inc();
metrics.http_requests_duration("GET", "/").observe(1.0);
metrics.current_users("service-1").set(10);
metrics.errors().inc();
```

#### Sample Output
```text
# HELP app_current_active_users The current number of active users.
# TYPE app_current_active_users gauge
app_current_active_users{host="localhost",port="8080",service="service-1"} 10
# HELP app_errors_total The total number of errors.
# TYPE app_errors_total counter
app_errors_total{host="localhost",port="8080"} 1
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
app_http_requests_total{host="localhost",method="GET",path="/",port="8080"} 1
```