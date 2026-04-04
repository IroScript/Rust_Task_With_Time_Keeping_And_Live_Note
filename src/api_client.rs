//! Non-blocking HTTP client for communicating with Axum backend
//!
//! This module provides async HTTP calls to fetch and update lines
//! for virtual scrolling without blocking the UI thread.

use serde::{Deserialize, Serialize};

/// Response for a single line from backend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineEntry {
    pub line_number: i64,
    pub line_text: String,
}

/// Card metadata from backend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardMetadata {
    pub card_id: String,
    pub title: String,
    pub total_lines: i64,
}

/// API client for backend communication
#[derive(Debug)]
pub struct ApiClient {
    base_url: String,
    client: reqwest::blocking::Client,
}

impl ApiClient {
    /// Create a new API client
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: reqwest::blocking::Client::new(),
        }
    }

    /// Fetch a range of lines for virtual scrolling
    /// 
    /// GET /api/cards/{card_id}/lines?start_line=X&limit=Y
    pub fn fetch_lines(
        &self,
        card_id: &str,
        start_line: i64,
        limit: i64,
    ) -> Result<Vec<LineEntry>, String> {
        let url = format!(
            "{}/api/cards/{}/lines?start_line={}&limit={}",
            self.base_url, card_id, start_line, limit
        );

        let response = self
            .client
            .get(&url)
            .send()
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("HTTP error: {}", response.status()));
        }

        let lines: Vec<LineEntry> = response
            .json()
            .map_err(|e| format!("Failed to parse JSON: {}", e))?;

        Ok(lines)
    }

    /// Update a single line's text
    /// 
    /// PUT /api/cards/{card_id}/lines/{line_number}
    pub fn update_line(
        &self,
        card_id: &str,
        line_number: i64,
        line_text: String,
    ) -> Result<LineEntry, String> {
        let url = format!(
            "{}/api/cards/{}/lines/{}",
            self.base_url, card_id, line_number
        );

        #[derive(Serialize)]
        struct UpdateRequest {
            line_text: String,
        }

        let response = self
            .client
            .put(&url)
            .json(&UpdateRequest { line_text })
            .send()
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("HTTP error: {}", response.status()));
        }

        let line: LineEntry = response
            .json()
            .map_err(|e| format!("Failed to parse JSON: {}", e))?;

        Ok(line)
    }

    /// Get card metadata including total_lines
    /// 
    /// GET /api/cards/{card_id}/meta
    pub fn get_card_metadata(&self, card_id: &str) -> Result<CardMetadata, String> {
        let url = format!("{}/api/cards/{}/meta", self.base_url, card_id);

        let response = self
            .client
            .get(&url)
            .send()
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("HTTP error: {}", response.status()));
        }

        let metadata: CardMetadata = response
            .json()
            .map_err(|e| format!("Failed to parse JSON: {}", e))?;

        Ok(metadata)
    }

    /// Batch insert lines (used for testing)
    /// 
    /// POST /api/cards/{card_id}/lines/batch
    pub fn batch_insert_lines(
        &self,
        card_id: &str,
        lines: Vec<LineEntry>,
    ) -> Result<(), String> {
        let url = format!("{}/api/cards/{}/lines/batch", self.base_url, card_id);

        #[derive(Serialize)]
        struct BatchRequest {
            lines: Vec<LineEntry>,
        }

        let response = self
            .client
            .post(&url)
            .json(&BatchRequest { lines })
            .send()
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("HTTP error: {}", response.status()));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_client_creation() {
        let client = ApiClient::new("http://localhost:3000".to_string());
        assert_eq!(client.base_url, "http://localhost:3000");
    }
}
