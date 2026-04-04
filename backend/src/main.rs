use anyhow::Result;
// use std::sync::Arc; // Removed - unused
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod db;
mod error;
mod models;
mod routes;
mod crdt;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,backend=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("🚀 Starting Pure Rust Backend with Axum API");
    tracing::info!("📊 Database: SQLite (local) or PostgreSQL (cloud) via Axum REST API");
    tracing::info!("🔒 All data access through Axum API endpoints - no direct DB access");

    // Load configuration
    let config = config::Config::from_env()?;
    tracing::info!("✅ Configuration loaded");

    // Connect to database (SQLite or PostgreSQL based on DATABASE_URL)
    let db_pool = db::init_db(&config.database_url)
        .await
        .map_err(|e| anyhow::anyhow!("Database initialization failed: {}", e))?;
    tracing::info!("✅ Connected to database via Axum API layer");

    // Create application state (already returns Arc<AppState>)
    let app_state = routes::AppState::new(db_pool);

    // Build router
    let app = routes::create_router(app_state, &config);

    // Start server
    let addr = format!("{}:{}", config.server_host, config.server_port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    
    tracing::info!("🎉 Server listening on http://{}", addr);
    tracing::info!("📡 WebSocket endpoint: ws://{}/ws/:document_id", addr);
    tracing::info!("🏥 Health check: http://{}/health", addr);

    axum::serve(listener, app)
        .await?;

    Ok(())
}
