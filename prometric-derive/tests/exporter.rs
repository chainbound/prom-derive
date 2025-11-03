use prometric::{Counter, exporter::ExporterBuilder};
use prometric_derive::metrics;

use http_body_util::{BodyExt, Empty};
use hyper::body::Bytes;
use hyper_util::{client::legacy::Client, rt::TokioExecutor};

#[metrics(scope = "test")]
struct TestMetrics {
    /// Test metric.
    #[metric]
    counter: Counter,
}

#[test]
fn test_exporter_thread() {
    let metrics = TestMetrics::default();

    metrics.counter().inc();

    ExporterBuilder::new().with_address("127.0.0.1:9090").with_namespace("app").install().unwrap();

    metrics.counter().inc();

    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();

    runtime.block_on(async {
        // Give the server a moment to start
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Create a client and make a request to the metrics endpoint
        let client = Client::builder(TokioExecutor::new()).build_http::<Empty<Bytes>>();

        let uri = "http://127.0.0.1:9090/".parse().unwrap();
        let response = client.get(uri).await.expect("Failed to make request");

        assert_eq!(response.status(), 200);

        // Read the response body
        let body_bytes =
            response.into_body().collect().await.expect("Failed to read response body").to_bytes();
        let body = String::from_utf8(body_bytes.to_vec()).expect("Invalid UTF-8");

        // Verify the metric is present with the global prefix
        assert!(body.contains("app_test_counter"));
        // Verify the counter value is 2 (incremented twice)
        assert!(body.contains("app_test_counter 2"));
    });
}

#[tokio::test]
async fn test_exporter_async() {
    let metrics = TestMetrics::default();

    metrics.counter().inc();

    ExporterBuilder::new()
        .with_address("127.0.0.1:9091")
        .with_path("/metrics")
        .with_namespace("app")
        .install()
        .unwrap();

    metrics.counter().inc();

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Create a client and make a request to the metrics endpoint
    let client = Client::builder(TokioExecutor::new()).build_http::<Empty<Bytes>>();

    let uri = "http://127.0.0.1:9091/metrics".parse().unwrap();
    let response = client.get(uri).await.expect("Failed to make request");

    assert_eq!(response.status(), 200);

    // Read the response body
    let body_bytes =
        response.into_body().collect().await.expect("Failed to read response body").to_bytes();
    let body = String::from_utf8(body_bytes.to_vec()).expect("Invalid UTF-8");

    // Verify the metric is present with the global prefix
    assert!(body.contains("app_test_counter"));
    // Verify the counter value is 2 (incremented twice)
    assert!(body.contains("app_test_counter 2"));
}
