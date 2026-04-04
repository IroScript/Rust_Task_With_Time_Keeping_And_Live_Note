use axum::{
    routing::{get, post},
    Router,
};
use dashmap::DashMap;
use std::sync::Arc;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
    compression::CompressionLayer,
};
use tokio::sync::broadcast as tokio_broadcast;

use crate::config::Config;
use crate::db::DbPool;
use shared::SyncMessage;

pub mod users;
pub mod documents;
pub mod websocket;
pub mod broadcast_mod;
pub mod settings;
pub mod lines;
pub mod cards;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: DbPool,
    pub crdt_docs: Arc<DashMap<String, Arc<tokio::sync::RwLock<yrs::Doc>>>>,
    pub connections: Arc<DashMap<String, Vec<String>>>, // document_id -> vec of connection_ids
    pub doc_channels: Arc<DashMap<String, tokio_broadcast::Sender<SyncMessage>>>,
}

impl AppState {
    pub fn new(db_pool: DbPool) -> Arc<Self> {
        Arc::new(Self {
            db_pool,
            crdt_docs: Arc::new(DashMap::new()),
            connections: Arc::new(DashMap::new()),
            doc_channels: Arc::new(DashMap::new()),
        })
    }
}

pub fn create_router(state: Arc<AppState>, config: &Config) -> Router {
    // CORS configuration
    let cors = CorsLayer::new()
        .allow_origin(config.cors_origin.parse::<axum::http::HeaderValue>().unwrap())
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        // Health check
        .route("/health", get(health_check))
        
        // User routes
        .route("/api/users", post(users::create_user))
        .route("/api/users/{id}", get(users::get_user))
        .route("/api/users/{id}", axum::routing::put(users::update_user))
        
        // Settings routes
        .route("/api/users/{user_id}/settings", post(settings::upsert_user_settings))
        .route("/api/users/{user_id}/settings", get(settings::get_user_settings))
        .route("/api/users/{user_id}/settings", axum::routing::put(settings::update_user_settings))
        
        // Document routes
        .route("/api/documents", post(documents::create_document))
        .route("/api/documents/{id}", get(documents::get_document))
        .route("/api/documents/{id}", axum::routing::put(documents::update_document))
        .route("/api/documents/{id}", axum::routing::delete(documents::delete_document))
        .route("/api/documents", get(documents::list_documents))
        
        // Card routes
        .route("/api/cards", get(cards::get_cards))
        .route("/api/cards", post(cards::create_card))
        
        // Line-level routes for virtual scrolling
        .route("/api/cards/{card_id}/lines", get(lines::get_card_lines))
        .route("/api/cards/{card_id}/lines/{line_number}", axum::routing::put(lines::update_card_line))
        .route("/api/cards/{card_id}/meta", get(lines::get_card_metadata))
        .route("/api/cards/{card_id}/lines/batch", post(lines::batch_insert_lines))
        
        // WebSocket route
        .route("/ws/{document_id}", get(websocket::ws_handler))
        
        .with_state(state)
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
}

async fn health_check() -> &'static str {
    "OK"
}
