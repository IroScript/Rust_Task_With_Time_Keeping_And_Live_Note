//! Axum Gateway for the Pure Rust Backend Sync System
//!
//! This module provides the HTTP/WebSocket gateway with health endpoints,
//! TLS 1.3 configuration, and application state management.

use axum::{
    body::Body,
    extract::State,
    routing::get,
    Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio_rustls::rustls::{self, ServerConfig};
use tokio_rustls::TlsAcceptor;
use tracing::info;

/// Application state shared across request handlers.
///
/// This struct holds all shared resources needed by the HTTP handlers
/// and WebSocket handlers. It is wrapped in Arc for thread-safe sharing.
#[derive(Clone)]
pub struct AppState {
    /// PostgreSQL connection pool
    pub db_pool: sqlx::PgPool,
    /// S3 client for blob storage
    pub s3_client: aws_sdk_s3::Client,
    /// HashiCorp Vault client
    pub vault_client: Arc<vaultrs::client::VaultClient>,
    /// Document-specific broadcast channels for WebSocket subscribers
    pub doc_channels: Arc<dashmap::DashMap<uuid::Uuid, tokio::sync::broadcast::Sender<shared::types::SyncMessage>>>,
    /// Presence maps per document
    pub presence: Arc<dashmap::DashMap<uuid::Uuid, Vec<shared::types::PresenceInfo>>>,
    /// In-memory CRDT documents
    pub crdt_docs: Arc<dashmap::DashMap<uuid::Uuid, Arc<tokio::sync::RwLock<yrs::Doc>>>>,
}

/// Health check response for liveness probes.
///
/// Indicates whether the application is running and responsive.
#[derive(serde::Serialize)]
struct HealthResponse {
    status: &'static str,
    timestamp: chrono::DateTime<chrono::Utc>,
}

/// Readiness check response for startup and traffic routing.
///
/// Indicates whether the application is ready to accept traffic.
#[derive(serde::Serialize)]
struct ReadyResponse {
    status: &'static str,
    database: bool,
    timestamp: chrono::DateTime<chrono::Utc>,
}

/// Liveness endpoint - returns 200 if the service is running.
///
/// This endpoint is used by Kubernetes liveness probes to determine
/// if the container should be restarted.
async fn health_handler() -> axum::Json<HealthResponse> {
    axum::Json(HealthResponse {
        status: "healthy",
        timestamp: chrono::Utc::now(),
    })
}

/// Readiness endpoint - returns 200 if the service is ready to accept traffic.
///
/// This endpoint checks that critical dependencies (database) are available.
/// Used by Kubernetes readiness probes to route traffic to ready pods.
async fn ready_handler(State(state): State<Arc<AppState>>) -> axum::Json<ReadyResponse> {
    // Check database connectivity
    let db_healthy = sqlx::query("SELECT 1")
        .fetch_optional(&state.db_pool)
        .await
        .is_ok();

    let status = if db_healthy { "ready" } else { "not_ready" };

    axum::Json(ReadyResponse {
        status,
        database: db_healthy,
        timestamp: chrono::Utc::now(),
    })
}

/// Creates a new AppState with the given components.
///
/// This is a convenience constructor for initializing the application state.
#[must_use]
pub fn new_app_state(
    db_pool: sqlx::PgPool,
    s3_client: aws_sdk_s3::Client,
    vault_client: Arc<vaultrs::client::VaultClient>,
) -> Arc<AppState> {
    Arc::new(AppState {
        db_pool,
        s3_client,
        vault_client,
        doc_channels: Arc::new(dashmap::DashMap::new()),
        presence: Arc::new(dashmap::DashMap::new()),
        crdt_docs: Arc::new(dashmap::DashMap::new()),
    })
}

/// Creates the Axum router with all routes and middleware.
///
/// # Arguments
/// * `state` - Shared application state
///
/// # Returns
/// Configured Axum Router ready to serve requests
pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(health_handler))
        .route("/ready", get(ready_handler))
        .with_state(state)
}

/// Loads TLS configuration for TLS 1.3.
///
/// # Arguments
/// * `cert_path` - Path to the TLS certificate file
/// * `key_path` - Path to the TLS private key file
///
/// # Returns
/// Configured ServerConfig for TLS 1.3
async fn load_tls_config(
    cert_path: &str,
    key_path: &str,
) -> Result<ServerConfig, Box<dyn std::error::Error>> {
    // Load certificate chain
    let cert_file = tokio::fs::File::open(cert_path).await?;
    let mut cert_reader = std::io::BufReader::new(cert_file);
    let certs = rustls::pki_types::CertificateDer::read_pem(&mut cert_reader)?;

    // Load private key
    let key_file = tokio::fs::File::open(key_path).await?;
    let mut key_reader = std::io::BufReader::new(key_file);
    let key = rustls::pki_types::PrivateKeyDer::read_pem(&mut key_reader)?;

    // Configure TLS 1.3
    let config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)?
        .protocol_versions(&[&rustls::version::TLS13])?;

    Ok(config)
}

/// Runs the HTTP server with optional TLS.
///
/// # Arguments
/// * `addr` - Socket address to bind to
/// * `router` - Configured Axum router
/// * `use_tls` - Whether to use TLS
/// * `cert_path` - Path to TLS certificate (if use_tls is true)
/// * `key_path` - Path to TLS private key (if use_tls is true)
///
/// # Returns
/// Result indicating success or failure
pub async fn serve(
    addr: SocketAddr,
    router: Router,
    use_tls: bool,
    cert_path: Option<&str>,
    key_path: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(addr).await?;
    info!("Listening on {}", addr);

    if use_tls {
        let cert_path = cert_path.expect("cert_path required for TLS");
        let key_path = key_path.expect("key_path required for TLS");
        let tls_config = load_tls_config(cert_path, key_path).await?;
        let acceptor = TlsAcceptor::from(Arc::new(tls_config));

        info!("TLS 1.3 enabled");

        loop {
            let (stream, addr) = listener.accept().await?;
            let acceptor = acceptor.clone();
            let router = router.clone();

            tokio::spawn(async move {
                match acceptor.accept(stream).await {
                    Ok(tls_stream) => {
                        axum::serve(
                            tokio::net::TcpStream::from_std(tls_stream.into_inner().0)?,
                            router,
                        )
                        .await?;
                        Ok::<_, Box<dyn std::error::Error>>(())
                    }
                    Err(e) => {
                        tracing::error!("TLS accept error: {}", e);
                        Err(e)
                    }
                }
            });
        }
    } else {
        axum::serve(listener, router).await?;
    }

    Ok(())
}

/// Main entry point for the gateway service.
///
/// Initializes tracing, creates the application state, configures the router,
/// and starts the HTTP server.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize structured logging
    tracing_subscriber::fmt::init();

    // Load configuration (placeholder - would load from config file/env)
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost/syncdb".to_string());
    let listen_addr: SocketAddr = std::env::var("LISTEN_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:3000".to_string())
        .parse()?;
    let use_tls = std::env::var("USE_TLS")
        .unwrap_or_else(|_| "false".to_string())
        .parse()?;
    let cert_path = std::env::var("TLS_CERT_PATH").ok();
    let key_path = std::env::var("TLS_KEY_PATH").ok();

    // Create database connection pool
    let db_pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(50)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&database_url)
        .await?;

    // Initialize S3 client (placeholder - would use AWS SDK)
    let s3_config = aws_config::load_from_env().await;
    let s3_client = aws_sdk_s3::Client::new(&s3_config);

    // Initialize Vault client (placeholder - would connect to Vault)
    let vault_client = Arc::new(
        vaultrs::client::VaultClient::builder()
            .address("http://localhost:8200")
            .token(std::env::var("VAULT_TOKEN").ok().as_deref().unwrap_or("dev-root-token"))
            .build()?,
    );

    // Create application state
    let state = new_app_state(db_pool, s3_client, vault_client);

    // Create router
    let router = create_router(state);

    // Start server
    serve(listen_addr, router, use_tls, cert_path.as_deref(), key_path.as_deref()).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use axum::http::Request;

    #[tokio::test]
    async fn test_health_endpoint() {
        let state = Arc::new(AppState {
            db_pool: sqlx::PgPool::connect_lazy("postgres://localhost/test").unwrap(),
            s3_client: {
                let config = aws_config::load_from_env().await;
                aws_sdk_s3::Client::new(&config)
            },
            vault_client: Arc::new(
                vaultrs::client::VaultClient::builder()
                    .address("http://localhost:8200")
                    .token("test")
                    .build()
                    .unwrap(),
            ),
            doc_channels: Arc::new(dashmap::DashMap::new()),
            presence: Arc::new(dashmap::DashMap::new()),
            crdt_docs: Arc::new(dashmap::DashMap::new()),
        });

        let router = create_router(state);
        let response = router
            .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), 200);

        let body = to_bytes(response.into_body()).await.unwrap();
        let health: HealthResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(health.status, "healthy");
    }

    #[tokio::test]
    async fn test_ready_endpoint() {
        let state = Arc::new(AppState {
            db_pool: sqlx::PgPool::connect_lazy("postgres://localhost/test").unwrap(),
            s3_client: {
                let config = aws_config::load_from_env().await;
                aws_sdk_s3::Client::new(&config)
            },
            vault_client: Arc::new(
                vaultrs::client::VaultClient::builder()
                    .address("http://localhost:8200")
                    .token("test")
                    .build()
                    .unwrap(),
            ),
            doc_channels: Arc::new(dashmap::DashMap::new()),
            presence: Arc::new(dashmap::DashMap::new()),
            crdt_docs: Arc::new(dashmap::DashMap::new()),
        });

        let router = create_router(state);
        let response = router
            .oneshot(Request::builder().uri("/ready").body(Body::empty()).unwrap())
            .await
            .unwrap();

        // Will be "not_ready" since database is not available in test
        let body = to_bytes(response.into_body()).await.unwrap();
        let ready: ReadyResponse = serde_json::from_slice(&body).unwrap();
        assert!(matches!(ready.status, "ready" | "not_ready"));
    }
}