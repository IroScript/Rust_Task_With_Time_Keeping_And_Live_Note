//! Line-level API endpoints for virtual scrolling
//!
//! These endpoints support fetching and updating individual lines
//! from the card_chunks table for efficient virtual scrolling.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{
    error::{AppError, AppResult},
    routes::AppState,
};

/// Response for a single line
#[derive(Debug, Serialize, Deserialize)]
pub struct LineEntry {
    pub line_number: i64,
    pub line_text: String,
}

/// Query parameters for fetching lines
#[derive(Debug, Deserialize)]
pub struct FetchLinesQuery {
    pub start_line: i64,
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_limit() -> i64 {
    50
}

/// Request body for updating a single line
#[derive(Debug, Deserialize)]
pub struct UpdateLineRequest {
    pub line_text: String,
}

/// Card metadata response
#[derive(Debug, Serialize)]
pub struct CardMetadata {
    pub card_id: String,
    pub title: String,
    pub total_lines: i64,
}

/// GET /api/cards/:card_id/lines
/// 
/// Fetches a range of lines for virtual scrolling.
/// Query params: ?start_line=500000&limit=50
/// 
/// Returns: JSON array of {line_number, line_text}
/// Response size: ~5-10 KB maximum
pub async fn get_card_lines(
    State(state): State<Arc<AppState>>,
    Path(card_id): Path<String>,
    Query(params): Query<FetchLinesQuery>,
) -> AppResult<Json<Vec<LineEntry>>> {
    // Validate parameters
    if params.start_line < 0 {
        return Err(AppError::InvalidInput(
            "start_line must be non-negative".to_string(),
        ));
    }

    if params.limit <= 0 || params.limit > 1000 {
        return Err(AppError::InvalidInput(
            "limit must be between 1 and 1000".to_string(),
        ));
    }

    // Fetch lines from database
    let lines = sqlx::query_as::<_, (i64, String)>(
        r#"
        SELECT line_number, line_text
        FROM card_chunks
        WHERE card_id = $1 AND line_number >= $2
        ORDER BY line_number ASC
        LIMIT $3
        "#,
    )
    .bind(&card_id)
    .bind(params.start_line)
    .bind(params.limit)
    .fetch_all(&state.db_pool)
    .await?;

    // Convert to LineEntry structs
    let entries: Vec<LineEntry> = lines
        .into_iter()
        .map(|(line_number, line_text)| LineEntry {
            line_number,
            line_text,
        })
        .collect();

    tracing::debug!(
        card_id = %card_id,
        start_line = params.start_line,
        count = entries.len(),
        "Fetched lines for virtual scrolling"
    );

    Ok(Json(entries))
}

/// PUT /api/cards/:card_id/lines/:line_number
/// 
/// Updates a single line's text.
/// Body: { "line_text": "updated content" }
pub async fn update_card_line(
    State(state): State<Arc<AppState>>,
    Path((card_id, line_number)): Path<(String, i64)>,
    Json(payload): Json<UpdateLineRequest>,
) -> AppResult<Json<LineEntry>> {
    // Validate line number
    if line_number < 0 {
        return Err(AppError::InvalidInput(
            "line_number must be non-negative".to_string(),
        ));
    }

    // Update the line
    let result = sqlx::query(
        r#"
        UPDATE card_chunks
        SET line_text = $1
        WHERE card_id = $2 AND line_number = $3
        "#,
    )
    .bind(&payload.line_text)
    .bind(&card_id)
    .bind(line_number)
    .execute(&state.db_pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!(
            "Line {} not found for card {}",
            line_number, card_id
        )));
    }

    tracing::info!(
        card_id = %card_id,
        line_number = line_number,
        "Updated line text"
    );

    Ok(Json(LineEntry {
        line_number,
        line_text: payload.line_text,
    }))
}

/// GET /api/cards/:card_id/meta
/// 
/// Returns card metadata including total_lines.
/// Used by frontend to calculate scrollbar height.
pub async fn get_card_metadata(
    State(state): State<Arc<AppState>>,
    Path(card_id): Path<String>,
) -> AppResult<Json<CardMetadata>> {
    // Fetch card metadata
    let card = sqlx::query_as::<_, (String, String, i64)>(
        r#"
        SELECT id, title, total_lines
        FROM cards
        WHERE id = $1
        "#,
    )
    .bind(&card_id)
    .fetch_optional(&state.db_pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Card {} not found", card_id)))?;

    Ok(Json(CardMetadata {
        card_id: card.0,
        title: card.1,
        total_lines: card.2,
    }))
}

/// POST /api/cards/:card_id/lines/batch
/// 
/// Batch insert lines (used by CLI ingestion tool).
/// Body: { "lines": [{"line_number": 1, "line_text": "..."}, ...] }
#[derive(Debug, Deserialize)]
pub struct BatchInsertRequest {
    pub lines: Vec<LineEntry>,
}

pub async fn batch_insert_lines(
    State(state): State<Arc<AppState>>,
    Path(card_id): Path<String>,
    Json(payload): Json<BatchInsertRequest>,
) -> AppResult<StatusCode> {
    if payload.lines.is_empty() {
        return Err(AppError::InvalidInput("lines array cannot be empty".to_string()));
    }

    if payload.lines.len() > 10000 {
        return Err(AppError::InvalidInput(
            "Maximum 10,000 lines per batch".to_string(),
        ));
    }

    // Begin transaction
    let mut tx = state.db_pool.begin().await?;

    // Insert all lines in batch
    for line in &payload.lines {
        sqlx::query(
            r#"
            INSERT INTO card_chunks (card_id, line_number, line_text)
            VALUES ($1, $2, $3)
            "#,
        )
        .bind(&card_id)
        .bind(line.line_number)
        .bind(&line.line_text)
        .execute(&mut *tx)
        .await?;
    }

    // Update total_lines count
    let max_line = payload.lines.iter().map(|l| l.line_number).max().unwrap_or(0);
    sqlx::query(
        r#"
        UPDATE cards
        SET total_lines = $1, updated_at = datetime('now')
        WHERE id = $2
        "#,
    )
    .bind(max_line + 1) // total_lines is count, not max index
    .bind(&card_id)
    .execute(&mut *tx)
    .await?;

    // Commit transaction
    tx.commit().await?;

    tracing::info!(
        card_id = %card_id,
        count = payload.lines.len(),
        "Batch inserted lines"
    );

    Ok(StatusCode::CREATED)
}

