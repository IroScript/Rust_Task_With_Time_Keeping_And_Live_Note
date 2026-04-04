use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{
    error::{AppError, AppResult},
    routes::AppState,
};

#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_per_page")]
    pub per_page: i64,
}

fn default_page() -> i64 {
    1
}

fn default_per_page() -> i64 {
    50
}

#[derive(Debug, Serialize)]
pub struct CardListResponse {
    pub cards: Vec<CardSummary>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
}

#[derive(Debug, Serialize)]
pub struct CardSummary {
    pub id: String,
    pub title: String,
    pub total_lines: i64,
    pub created_at: String,
    pub updated_at: String,
}

/// GET /api/cards - Get paginated list of cards
pub async fn get_cards(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PaginationParams>,
) -> AppResult<Json<CardListResponse>> {
    // Calculate offset
    let offset = (params.page - 1) * params.per_page;

    // Get total count
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM cards")
        .fetch_one(&state.db_pool)
        .await?;

    // Get paginated cards
    let cards = sqlx::query_as::<_, (String, String, i64, String, String)>(
        "SELECT id, title, total_lines, created_at, updated_at 
         FROM cards 
         ORDER BY updated_at DESC 
         LIMIT $1 OFFSET $2",
    )
    .bind(params.per_page)
    .bind(offset)
    .fetch_all(&state.db_pool)
    .await?
    .into_iter()
    .map(|(id, title, total_lines, created_at, updated_at)| CardSummary {
        id,
        title,
        total_lines,
        created_at,
        updated_at,
    })
    .collect();

    Ok(Json(CardListResponse {
        cards,
        total,
        page: params.page,
        per_page: params.per_page,
    }))
}

/// POST /api/cards - Create a new card
#[derive(Debug, Deserialize)]
pub struct CreateCardRequest {
    pub title: String,
}

pub async fn create_card(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateCardRequest>,
) -> AppResult<Json<CardSummary>> {
    let now = chrono::Utc::now().to_rfc3339();
    let card_id = uuid::Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO cards (id, title, total_lines, created_at, updated_at) 
         VALUES ($1, $2, 0, $3, $4)",
    )
    .bind(&card_id)
    .bind(&payload.title)
    .bind(&now)
    .bind(&now)
    .execute(&state.db_pool)
    .await?;

    Ok(Json(CardSummary {
        id: card_id,
        title: payload.title,
        total_lines: 0,
        created_at: now.clone(),
        updated_at: now,
    }))
}
