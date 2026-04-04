use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: String,  // Changed from Uuid to String for SQLite compatibility
    pub name: String,
    pub email: String,
    pub country_code: String,
    pub company_name: String,
    pub created_at: String,  // SQLite stores as TEXT
    pub updated_at: String,  // SQLite stores as TEXT
}

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub name: String,
    pub email: String,
    pub country_code: String,
    pub company_name: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub name: Option<String>,
    pub email: Option<String>,
    pub country_code: Option<String>,
    pub company_name: Option<String>,
}
