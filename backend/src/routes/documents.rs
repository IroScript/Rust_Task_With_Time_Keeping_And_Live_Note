use std::sync::Arc;
use axum::{extract::{Path, Query, State}, Json, http::StatusCode};
use serde::Deserialize;
use yrs::{Transact, Text, ReadTxn};
use yrs::updates::encoder::Encode;

use crate::{
    error::{AppResult, AppError},
    models::{Document, CreateDocumentRequest, UpdateDocumentRequest},
    routes::AppState,
};

/// Create a new document
pub async fn create_document(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateDocumentRequest>,
) -> AppResult<(StatusCode, Json<Document>)> {
    // Validate input
    if payload.title.trim().is_empty() {
        return Err(AppError::InvalidInput("Title cannot be empty".to_string()));
    }

    // Verify user exists
    let user_id_ref = &payload.user_id;
    let user_exists = sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM users WHERE id = $1)")
        .bind(user_id_ref)
        .fetch_one(&state.db_pool)
        .await?;

    if !user_exists {
        return Err(AppError::NotFound(format!("User {} not found", payload.user_id)));
    }

    // Initialize CRDT document if initial content provided
    let crdt_content = if let Some(initial_text) = payload.initial_content {
        let doc = yrs::Doc::new();
        let text = doc.get_or_insert_text("content");
        let mut txn = doc.transact_mut();
        text.insert(&mut txn, 0, &initial_text);
        drop(txn);
        
        // Encode CRDT state to bytes
        let state_vector = doc.transact().state_vector().encode_v1();
        Some(state_vector)
    } else {
        None
    };

    // Insert document into database
    let document = sqlx::query_as::<_, Document>(
        r#"
        INSERT INTO documents (user_id, title, content, crdt_version)
        VALUES ($1, $2, $3, 0)
        RETURNING id, user_id, title, content, crdt_version, created_at, updated_at
        "#,
    )
    .bind(&payload.user_id)
    .bind(&payload.title)
    .bind(crdt_content)
    .fetch_one(&state.db_pool)
    .await?;

    tracing::info!("Created document: {} ({})", document.title, document.id);

    Ok((StatusCode::CREATED, Json(document)))
}

/// Get document by ID
pub async fn get_document(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> AppResult<Json<Document>> {
    let document = sqlx::query_as::<_, Document>(
        r#"
        SELECT id, user_id, title, content, crdt_version, created_at, updated_at
        FROM documents
        WHERE id = $1
        "#,
    )
    .bind(&id)
    .fetch_optional(&state.db_pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Document {} not found", id)))?;

    Ok(Json(document))
}

/// Update document metadata (title only, content updated via WebSocket)
pub async fn update_document(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateDocumentRequest>,
) -> AppResult<Json<Document>> {
    // Check if document exists
    let existing_doc = sqlx::query_as::<_, Document>(
        "SELECT id, user_id, title, content, crdt_version, created_at, updated_at FROM documents WHERE id = $1",
    )
    .bind(&id)
    .fetch_optional(&state.db_pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Document {} not found", id)))?;

    // Update only title if provided
    let title = payload.title.unwrap_or(existing_doc.title);

    let updated_doc = sqlx::query_as::<_, Document>(
        r#"
        UPDATE documents
        SET title = $1, updated_at = now()
        WHERE id = $2
        RETURNING id, user_id, title, content, crdt_version, created_at, updated_at
        "#,
    )
    .bind(&title)
    .bind(&id)
    .fetch_one(&state.db_pool)
    .await?;

    tracing::info!("Updated document: {} ({})", updated_doc.title, updated_doc.id);

    Ok(Json(updated_doc))
}

/// Delete document
pub async fn delete_document(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    let result = sqlx::query("DELETE FROM documents WHERE id = $1")
        .bind(&id)
        .execute(&state.db_pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("Document {} not found", &id)));
    }

    tracing::info!("Deleted document: {}", &id);

    Ok(Json(serde_json::json!({
        "message": "Document deleted successfully",
        "id": id
    })))
}

#[derive(Debug, Deserialize)]
pub struct ListDocumentsQuery {
    user_id: Option<String>,
    limit: Option<i64>,
    offset: Option<i64>,
}

/// List documents with optional filtering
pub async fn list_documents(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListDocumentsQuery>,
) -> AppResult<Json<Vec<Document>>> {
    let limit = params.limit.unwrap_or(50).min(100); // Max 100 documents per request
    let offset = params.offset.unwrap_or(0);

    let documents = if let Some(user_id) = params.user_id {
        // Filter by user_id
        sqlx::query_as::<_, Document>(
            r#"
            SELECT id, user_id, title, content, crdt_version, created_at, updated_at
            FROM documents
            WHERE user_id = $1
            ORDER BY updated_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(&user_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&state.db_pool)
        .await?
    } else {
        // Get all documents
        sqlx::query_as::<_, Document>(
            r#"
            SELECT id, user_id, title, content, crdt_version, created_at, updated_at
            FROM documents
            ORDER BY updated_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&state.db_pool)
        .await?
    };

    Ok(Json(documents))
}
