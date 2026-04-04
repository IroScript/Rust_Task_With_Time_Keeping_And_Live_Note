use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Document {
    pub id: String,  // Changed from Uuid to String for SQLite
    pub user_id: String,  // Changed from Uuid to String
    pub title: String,
    #[sqlx(default)]
    pub content: Option<Vec<u8>>,
    pub crdt_version: i64,
    pub created_at: String,  // SQLite stores as TEXT
    pub updated_at: String,  // SQLite stores as TEXT
}

#[derive(Debug, Deserialize)]
pub struct CreateDocumentRequest {
    pub user_id: String,  // Changed from Uuid to String
    pub title: String,
    pub initial_content: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateDocumentRequest {
    pub title: Option<String>,
}
