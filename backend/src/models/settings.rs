use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserSettings {
    pub id: String,  // Changed from Uuid to String for SQLite
    pub user_id: String,  // Changed from Uuid to String
    pub settings_data: String,  // SQLite stores JSON as TEXT
    pub created_at: String,  // SQLite stores as TEXT
    pub updated_at: String,  // SQLite stores as TEXT
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSettingsRequest {
    pub settings_data: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateSettingsRequest {
    pub settings_data: serde_json::Value,
}

// Settings structure that matches frontend
#[derive(Debug, Serialize, Deserialize)]
pub struct AppSettings {
    pub theme: ThemeConfig,
    pub text_style: TextStyleConfig,
    pub interval_secs: u64,
    pub single_quote_mode: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ThemeConfig {
    pub mode: String, // "Gradient" or "Solid"
    pub gradient_angle: i32,
    pub gradient_colors: Vec<u32>,
    pub solid_color: u32,
    pub apply_to_entire_window: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TextStyleConfig {
    pub main_text_size: f32,
    pub sub_text_size: f32,
    pub main_text_color: u32,
    pub sub_text_color: u32,
    pub panel_text_color: u32,
    pub main_line_gap: f32,
    pub sub_line_gap: f32,
    pub between_gap: f32,
}
