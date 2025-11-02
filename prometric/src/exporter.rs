use std::{net::SocketAddr, thread};

use hyper::{
    Request, Response, body::Incoming, header::CONTENT_TYPE, server::conn::http1,
    service::service_fn,
};
use hyper_util::rt::TokioIo;
use prometheus::{Encoder, TextEncoder};

pub struct ExporterBuilder {
    registry: Option<prometheus::Registry>,
    address: SocketAddr,
    path: Option<String>,
    global_prefix: Option<String>,
}

impl Default for ExporterBuilder {
    fn default() -> Self {
        Self {
            registry: None,
            address: "0.0.0.0:9090".parse().unwrap(),
            path: None,
            global_prefix: None,
        }
    }
}

pub enum ExporterError {
    BindError(std::io::Error),
    InvalidPath(String),
}

impl std::error::Error for ExporterError {}

impl std::fmt::Display for ExporterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BindError(e) => write!(f, "Failed to bind to address: {}", e),
            Self::InvalidPath(path) => write!(f, "Invalid path: {}", path),
        }
    }
}

impl From<std::io::Error> for ExporterError {
    fn from(e: std::io::Error) -> Self {
        Self::BindError(e)
    }
}

impl std::fmt::Debug for ExporterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl ExporterBuilder {
    /// Create a new exporter with the default registry and socket address.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the socket address for the exporter.
    ///
    /// # Panics
    /// Panics if the address is malformed.
    pub fn with_address(mut self, address: impl Into<String>) -> Self {
        let address = address.into();
        self.address = address.parse().unwrap();
        self
    }

    /// Set the path for the exporter.
    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        let path = path.into();
        self.path = Some(path);
        self
    }

    /// Set the global prefix for the exporter. This will be prepended to all metric names.
    pub fn with_global_prefix(mut self, global_prefix: impl Into<String>) -> Self {
        let global_prefix = global_prefix.into();
        self.global_prefix = Some(global_prefix);
        self
    }

    /// Set the registry for the exporter.
    pub fn with_registry(mut self, registry: prometheus::Registry) -> Self {
        self.registry = Some(registry);
        self
    }

    fn path(&self) -> Result<String, ExporterError> {
        if let Some(path) = self.path.clone() {
            if path.is_empty() {
                return Err(ExporterError::InvalidPath(path));
            }

            if !path.starts_with('/') {
                return Err(ExporterError::InvalidPath(path));
            }

            if path.ends_with('/') {
                return Err(ExporterError::InvalidPath(path));
            }

            Ok(path)
        } else {
            Ok("/".to_owned())
        }
    }

    /// Install the exporter and start serving metrics.
    ///
    /// # Behavior
    /// - If a Tokio runtime is available, use it to spawn the listener.
    /// - Otherwise, spawn a new single-threaded Tokio runtime on a thread, and spawn the listener there.
    pub fn install(self) -> Result<(), ExporterError> {
        let path = self.path()?;
        let address = self.address;
        let registry = self
            .registry
            .unwrap_or_else(|| prometheus::default_registry().clone());

        let serve = serve(address, registry, path, self.global_prefix);

        // If the runtime is available, use it to spawn the listener. Otherwise,
        // create a new single-threaded runtime and spawn the listener there.
        if let Ok(runtime) = tokio::runtime::Handle::try_current() {
            runtime.spawn(serve);
        } else {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()?;

            thread::spawn(move || {
                runtime
                    .block_on(serve)
                    .unwrap_or_else(|e| panic!("server error: {:?}", e));
            });
        }

        Ok(())
    }
}

async fn serve(
    addr: SocketAddr,
    registry: prometheus::Registry,
    path: String,
    global_prefix: Option<String>,
) -> Result<(), ExporterError> {
    let listener = tokio::net::TcpListener::bind(addr).await?;
    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);

        let registry = registry.clone();
        let path = path.clone();
        let global_prefix = global_prefix.clone();

        let service = service_fn(move |req| {
            serve_req(req, registry.clone(), path.clone(), global_prefix.clone())
        });
        if let Err(err) = http1::Builder::new().serve_connection(io, service).await {
            eprintln!("server error: {:?}", err);
        };
    }
}

async fn serve_req(
    req: Request<Incoming>,
    registry: prometheus::Registry,
    path: String,
    global_prefix: Option<String>,
) -> Result<Response<String>, Box<dyn std::error::Error + Send + Sync>> {
    let encoder = TextEncoder::new();
    let mut metrics = registry.gather();

    if req.uri().path() != path {
        return Ok(Response::builder()
            .status(404)
            .body("Not Found".to_string())?);
    }

    // Set the global prefix for the metrics
    if let Some(prefix) = global_prefix {
        metrics.iter_mut().for_each(|metric| {
            metric.name.as_mut().map(|name| {
                name.insert(0, '_');
                name.insert_str(0, &prefix);
            });
        });
    }

    let body = encoder.encode_to_string(&metrics)?;

    let response = Response::builder()
        .status(200)
        .header(CONTENT_TYPE, encoder.format_type())
        .body(body)?;

    Ok(response)
}
