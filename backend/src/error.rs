use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    // Comment out unused variants to avoid warnings
    // #[error("Internal server error: {0}")]
    // Internal(String),

    // #[error("CRDT error: {0}")]
    // Crdt(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::Database(ref e) => {
                tracing::error!("Database error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Database error")
            }
            AppError::NotFound(ref msg) => (StatusCode::NOT_FOUND, msg.as_str()),
            AppError::InvalidInput(ref msg) => (StatusCode::BAD_REQUEST, msg.as_str()),
            // Commented out unused variants
            // AppError::Internal(ref msg) => {
            //     tracing::error!("Internal error: {}", msg);
            //     (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            // }
            // AppError::Crdt(ref msg) => {
            //     tracing::error!("CRDT error: {}", msg);
            //     (StatusCode::INTERNAL_SERVER_ERROR, "CRDT synchronization error")
            // }
        };

        let body = Json(json!({
            "error": error_message,
            "details": self.to_string(),
        }));

        (status, body).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;
