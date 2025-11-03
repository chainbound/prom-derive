# `prometric`

[![Lints](https://github.com/chainbound/prometric/actions/workflows/lint.yml/badge.svg?branch=main)](https://github.com/chainbound/prometric/actions/workflows/lint.yml)
[![Tests](https://github.com/chainbound/prometric/actions/workflows/test.yml/badge.svg?branch=main)](https://github.com/chainbound/prometric/actions/workflows/test.yml)
[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/chainbound/prometric)

A library for ergonomically generating and using embedded Prometheus metrics in Rust.

Inspired by [metrics-derive](https://github.com/ithacaxyz/metrics-derive), but works directly with [prometheus](https://docs.rs/prometheus/latest/prometheus)
instead of [metrics](https://docs.rs/metrics/latest/metrics), and supports dynamic labels.

| Crate              | crates.io                                                                                                       | docs.rs                                                                                    |
| ------------------ | --------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------ |
| `prometric`        | [![crates.io](https://img.shields.io/crates/v/prometric.svg)](https://crates.io/crates/prometric)               | [![docs.rs](https://docs.rs/prometric/badge.svg)](https://docs.rs/prometric)               |
| `prometric-derive` | [![crates.io](https://img.shields.io/crates/v/prometric-derive.svg)](https://crates.io/crates/prometric-derive) | [![docs.rs](https://docs.rs/prometric-derive/badge.svg)](https://docs.rs/prometric-derive) |

## Usage

### Basic Usage

```rust
use prometric_derive::metrics;
use prometric::{Counter, Gauge, Histogram};

// The `scope` attribute is used to set the prefix for the metric names in this struct.
#[metrics(scope = "app")]
struct AppMetrics {
    /// The total number of HTTP requests.
    #[metric(rename = "http_requests_total", labels = ["method", "path"])]
    http_requests: Counter,

    // For histograms, the `buckets` attribute is optional. It will default to [prometheus::DEFAULT_BUCKETS] if not provided.
    /// The duration of HTTP requests.
    #[metric(labels = ["method", "path"], buckets = [0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0])]
    http_requests_duration: Histogram,

    /// This doc comment will be overwritten by the `help` attribute.
    #[metric(rename = "current_active_users", labels = ["service"], help = "The current number of active users.")]
    current_users: Gauge,

    /// The balance of the account, in dollars. Uses a floating point number.
    #[metric(rename = "account_balance", labels = ["account_id"])]
    account_balance: Gauge<f64>,

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
metrics.account_balance("1234567890").set(-12.2);
metrics.errors().inc();
```

### Static Metrics

You can also generate a static `LazyLock` instance by using the `static` attribute. When enabled, the builder methods and `Default` implementation are made private, ensuring the only way to access the metrics is through the static instance:

```rust
use prometric_derive::metrics;
use prometric::{Counter, Gauge};

#[metrics(scope = "app", static)]
struct AppMetrics {
    /// The total number of requests.
    #[metric(labels = ["method"])]
    requests: Counter,

    /// The current number of active connections.
    #[metric]
    active_connections: Gauge,
}

// Use the static directly (the name is APP_METRICS in SCREAMING_SNAKE_CASE)
APP_METRICS.requests("GET").inc();
APP_METRICS.active_connections().set(10);

// The following would not compile:
// let metrics = AppMetrics::builder();  // Error: builder() is private
// let metrics = AppMetrics::default();   // Error: Default is not implemented
```

### Exporting Metrics

An HTTP exporter is provided by [`prometric::exporter::ExporterBuilder`]. Usage:

```rust
use prometric::exporter::ExporterBuilder;

ExporterBuilder::new()
    // Specify the address to listen on
    .with_address("127.0.0.1:9090")
    // Set the global namespace for the metrics (usually the name of the application)
    .with_namespace("exporter")
    // Install the exporter. This will start an HTTP server and serve metrics on the specified
    // address.
    .install()
    .expect("Failed to install exporter");
```

#### Sample Output

```text
# HELP app_account_balance The balance of the account, in dollars. Uses a floating point number.
# TYPE app_account_balance gauge
app_account_balance{account_id="1234567890",host="localhost",port="8080"} -12.2

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
app_http_requests_duration_bucket{host="localhost",method="GET",path="/",port="8080",le="+Inf"} 1
app_http_requests_duration_sum{host="localhost",method="GET",path="/",port="8080"} 1
app_http_requests_duration_count{host="localhost",method="GET",path="/",port="8080"} 1

# HELP app_http_requests_total The total number of HTTP requests.
# TYPE app_http_requests_total counter
app_http_requests_total{host="localhost",method="GET",path="/",port="8080"} 2
app_http_requests_total{host="localhost",method="POST",path="/",port="8080"} 2
```
