use std::sync::Arc;
use axum::{extract::{Path, State}, Json, http::StatusCode};

use crate::{
    error::{AppResult, AppError},
    models::{User, CreateUserRequest, UpdateUserRequest},
    routes::AppState,
};

/// Create a new user (no authentication required for MVP)
pub async fn create_user(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateUserRequest>,
) -> AppResult<(StatusCode, Json<User>)> {
    // Validate input
    if payload.name.trim().is_empty() {
        return Err(AppError::InvalidInput("Name cannot be empty".to_string()));
    }
    if payload.email.trim().is_empty() {
        return Err(AppError::InvalidInput("Email cannot be empty".to_string()));
    }
    if payload.country_code.trim().is_empty() {
        return Err(AppError::InvalidInput("Country code cannot be empty".to_string()));
    }
    if payload.company_name.trim().is_empty() {
        return Err(AppError::InvalidInput("Company name cannot be empty".to_string()));
    }

    // Insert user into database
    let user = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (name, email, country_code, company_name)
        VALUES ($1, $2, $3, $4)
        RETURNING id, name, email, country_code, company_name, created_at, updated_at
        "#,
    )
    .bind(&payload.name)
    .bind(&payload.email)
    .bind(&payload.country_code)
    .bind(&payload.company_name)
    .fetch_one(&state.db_pool)
    .await?;

    tracing::info!("Created user: {} ({})", user.name, user.id);

    Ok((StatusCode::CREATED, Json(user)))
}

/// Get user by ID
pub async fn get_user(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> AppResult<Json<User>> {
    let user = sqlx::query_as::<_, User>(
        r#"
        SELECT id, name, email, country_code, company_name, created_at, updated_at
        FROM users
        WHERE id = $1
        "#,
    )
    .bind(&id)
    .fetch_optional(&state.db_pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("User {} not found", id)))?;

    Ok(Json(user))
}

/// Update user information
pub async fn update_user(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateUserRequest>,
) -> AppResult<Json<User>> {
    // Check if user exists
    let existing_user = sqlx::query_as::<_, User>(
        "SELECT id, name, email, country_code, company_name, created_at, updated_at FROM users WHERE id = $1",
    )
    .bind(&id)
    .fetch_optional(&state.db_pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("User {} not found", id)))?;

    // Update only provided fields
    let name = payload.name.unwrap_or(existing_user.name);
    let email = payload.email.unwrap_or(existing_user.email);
    let country_code = payload.country_code.unwrap_or(existing_user.country_code);
    let company_name = payload.company_name.unwrap_or(existing_user.company_name);

    let updated_user = sqlx::query_as::<_, User>(
        r#"
        UPDATE users
        SET name = $1, email = $2, country_code = $3, company_name = $4, updated_at = now()
        WHERE id = $5
        RETURNING id, name, email, country_code, company_name, created_at, updated_at
        "#,
    )
    .bind(&name)
    .bind(&email)
    .bind(&country_code)
    .bind(&company_name)
    .bind(&id)
    .fetch_one(&state.db_pool)
    .await?;

    tracing::info!("Updated user: {} ({})", updated_user.name, updated_user.id);

    Ok(Json(updated_user))
}
