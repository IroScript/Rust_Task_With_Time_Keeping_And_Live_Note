use std::sync::Arc;
use axum::{extract::{Path, State}, Json, http::StatusCode};

use crate::{
    error::{AppResult, AppError},
    models::settings::{UserSettings, CreateSettingsRequest, UpdateSettingsRequest},
    routes::AppState,
};

/// Create or update user settings (upsert)
pub async fn upsert_user_settings(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<String>,
    Json(payload): Json<CreateSettingsRequest>,
) -> AppResult<(StatusCode, Json<UserSettings>)> {
    // Validate that settings_data is not empty
    if payload.settings_data.is_null() {
        return Err(AppError::InvalidInput("Settings data cannot be null".to_string()));
    }

    // Convert JSON to string for SQLite
    let settings_json = serde_json::to_string(&payload.settings_data)
        .map_err(|e| AppError::InvalidInput(format!("Invalid JSON: {}", e)))?;

    // Upsert settings (insert or update if exists)
    let settings = sqlx::query_as::<_, UserSettings>(
        r#"
        INSERT INTO user_settings (user_id, settings_data)
        VALUES ($1, $2)
        ON CONFLICT (user_id) 
        DO UPDATE SET 
            settings_data = EXCLUDED.settings_data,
            updated_at = NOW()
        RETURNING id, user_id, settings_data, created_at, updated_at
        "#,
    )
    .bind(&user_id)
    .bind(&settings_json)
    .fetch_one(&state.db_pool)
    .await?;

    tracing::info!("Upserted settings for user: {}", &user_id);

    Ok((StatusCode::OK, Json(settings)))
}

/// Get user settings by user ID
pub async fn get_user_settings(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<String>,
) -> AppResult<Json<UserSettings>> {
    let settings = sqlx::query_as::<_, UserSettings>(
        r#"
        SELECT id, user_id, settings_data, created_at, updated_at
        FROM user_settings
        WHERE user_id = $1
        "#,
    )
    .bind(&user_id)
    .fetch_optional(&state.db_pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Settings for user {} not found", user_id)))?;

    Ok(Json(settings))
}

/// Update user settings
pub async fn update_user_settings(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<String>,
    Json(payload): Json<UpdateSettingsRequest>,
) -> AppResult<Json<UserSettings>> {
    // Validate that settings_data is not empty
    if payload.settings_data.is_null() {
        return Err(AppError::InvalidInput("Settings data cannot be null".to_string()));
    }

    // Convert JSON to string for SQLite
    let settings_json = serde_json::to_string(&payload.settings_data)
        .map_err(|e| AppError::InvalidInput(format!("Invalid JSON: {}", e)))?;

    // Update settings
    let settings = sqlx::query_as::<_, UserSettings>(
        r#"
        UPDATE user_settings
        SET settings_data = $1, updated_at = NOW()
        WHERE user_id = $2
        RETURNING id, user_id, settings_data, created_at, updated_at
        "#,
    )
    .bind(&settings_json)
    .bind(&user_id)
    .fetch_optional(&state.db_pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Settings for user {} not found", user_id)))?;

    tracing::info!("Updated settings for user: {}", &user_id);

    Ok(Json(settings))
}