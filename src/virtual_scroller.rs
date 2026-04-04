//! Pure Rust Virtual Scrolling Component for egui
//!
//! This module implements virtual scrolling for billions of lines
//! while keeping memory usage under 30 MB.
//!
//! Architecture:
//! - Only 50-100 lines exist in memory at any time
//! - Lines are fetched on-demand via HTTP from backend
//! - Invisible spacing creates illusion of full document
//! - Debounced fetching prevents excessive HTTP requests

use crate::api_client::{ApiClient, LineEntry};
use egui::{Color32, RichText, ScrollArea, Ui, Vec2};
use std::time::{Duration, Instant};

/// Virtual scrolling viewer for massive text documents
#[derive(Debug)]
pub struct LiveNoteViewer {
    /// Card ID being viewed
    pub card_id: String,

    /// Total number of lines in the document
    pub total_lines: i64,

    /// Currently loaded lines (ALWAYS ≤ 150 entries)
    pub loaded_lines: Vec<LineEntry>,

    /// First line number in loaded_lines buffer
    pub loaded_start: i64,

    /// Last line number in loaded_lines buffer
    pub loaded_end: i64,

    /// Current scroll offset in pixels
    pub scroll_offset: f32,

    /// Height of each line in pixels (monospace font)
    pub line_height: f32,

    /// Number of lines visible on screen
    pub visible_count: usize,

    /// Whether we're currently fetching data
    pub is_fetching: bool,

    /// Last fetch time (for debouncing)
    pub last_fetch_time: Option<Instant>,

    /// Debounce delay in milliseconds
    pub debounce_delay: Duration,

    /// Line being edited (if any)
    pub editing_line: Option<i64>,

    /// Edit buffer for current line
    pub edit_buffer: String,

    /// API client for backend communication
    api_client: ApiClient,

    /// Font size for rendering
    pub font_size: f32,

    /// Text color
    pub text_color: Color32,

    /// Fetch buffer size (lines to prefetch above/below visible area)
    pub fetch_buffer: i64,
}

impl LiveNoteViewer {
    /// Create a new virtual scroller
    pub fn new(card_id: String, backend_url: String) -> Self {
        Self {
            card_id,
            total_lines: 0,
            loaded_lines: Vec::new(),
            loaded_start: 0,
            loaded_end: 0,
            scroll_offset: 0.0,
            line_height: 20.0, // Will be calculated based on font
            visible_count: 0,
            is_fetching: false,
            last_fetch_time: None,
            debounce_delay: Duration::from_millis(100),
            editing_line: None,
            edit_buffer: String::new(),
            api_client: ApiClient::new(backend_url),
            font_size: 14.0,
            text_color: Color32::WHITE,
            fetch_buffer: 25,
        }
    }

    /// Initialize by fetching card metadata
    pub fn init(&mut self) -> Result<(), String> {
        let metadata = self.api_client.get_card_metadata(&self.card_id)?;
        self.total_lines = metadata.total_lines;
        Ok(())
    }

    /// Render the virtual scroller
    pub fn show(&mut self, ui: &mut Ui) {
        // Calculate geometry
        let available_height = ui.available_height();
        self.line_height = self.font_size * 1.5; // Line height with spacing
        self.visible_count = (available_height / self.line_height) as usize;

        // Create scroll area with virtual height
        let total_height = self.total_lines as f32 * self.line_height;

        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show_viewport(ui, |ui, viewport| {
                // Calculate which lines should be visible
                let first_visible_line = (viewport.min.y / self.line_height) as i64;
                let last_visible_line =
                    ((viewport.max.y / self.line_height) as i64).min(self.total_lines - 1);

                // Determine fetch range with buffer
                let fetch_start = (first_visible_line - self.fetch_buffer).max(0);
                let fetch_end = (last_visible_line + self.fetch_buffer).min(self.total_lines - 1);

                // Check if we need to fetch new data
                let needs_fetch = fetch_start < self.loaded_start
                    || fetch_end > self.loaded_end
                    || self.loaded_lines.is_empty();

                if needs_fetch && !self.is_fetching {
                    // Check debounce
                    let should_fetch = if let Some(last_time) = self.last_fetch_time {
                        last_time.elapsed() >= self.debounce_delay
                    } else {
                        true
                    };

                    if should_fetch {
                        self.fetch_lines(fetch_start, fetch_end);
                    }
                }

                // Allocate full virtual height
                ui.allocate_space(Vec2::new(ui.available_width(), total_height));

                // Add invisible spacing at top
                if self.loaded_start > 0 {
                    let top_spacing = self.loaded_start as f32 * self.line_height;
                    ui.add_space(top_spacing);
                }

                // Render loaded lines
                let loaded_lines = self.loaded_lines.clone();
                for line in &loaded_lines {
                    if line.line_number >= first_visible_line
                        && line.line_number <= last_visible_line
                    {
                        self.render_line(ui, line);
                    }
                }

                // Add invisible spacing at bottom
                if self.loaded_end < self.total_lines - 1 {
                    let remaining = self.total_lines - self.loaded_end - 1;
                    let bottom_spacing = remaining as f32 * self.line_height;
                    ui.add_space(bottom_spacing);
                }

                // Show loading indicator if fetching
                if self.is_fetching {
                    ui.label(
                        RichText::new("⏳ Loading...")
                            .color(Color32::YELLOW)
                            .size(12.0),
                    );
                }
            });
    }

    /// Render a single line
    fn render_line(&mut self, ui: &mut Ui, line: &LineEntry) {
        let is_editing = self.editing_line == Some(line.line_number);

        ui.horizontal(|ui| {
            // Line number (clickable)
            let line_num_label = ui.label(
                RichText::new(format!("{:6} │ ", line.line_number))
                    .color(Color32::DARK_GRAY)
                    .monospace()
                    .size(self.font_size),
            );

            if line_num_label.clicked() {
                // Start editing this line
                self.editing_line = Some(line.line_number);
                self.edit_buffer = line.line_text.clone();
            }

            // Line text (editable if selected)
            if is_editing {
                let response = ui.text_edit_singleline(&mut self.edit_buffer);

                // Save on Enter or focus loss
                if response.lost_focus() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    self.save_line(line.line_number);
                    self.editing_line = None;
                }

                // Cancel on Escape
                if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    self.editing_line = None;
                }
            } else {
                let text_label = ui.label(
                    RichText::new(&line.line_text)
                        .color(self.text_color)
                        .monospace()
                        .size(self.font_size),
                );

                // Double-click to edit
                if text_label.double_clicked() {
                    self.editing_line = Some(line.line_number);
                    self.edit_buffer = line.line_text.clone();
                }
            }
        });
    }

    /// Fetch lines from backend
    fn fetch_lines(&mut self, start: i64, end: i64) {
        self.is_fetching = true;
        self.last_fetch_time = Some(Instant::now());

        let limit = (end - start + 1).min(150); // Cap at 150 lines

        match self.api_client.fetch_lines(&self.card_id, start, limit) {
            Ok(lines) => {
                // Drop old data (free memory)
                self.loaded_lines.clear();
                self.loaded_lines = lines;

                // Update range
                if let (Some(first), Some(last)) =
                    (self.loaded_lines.first(), self.loaded_lines.last())
                {
                    self.loaded_start = first.line_number;
                    self.loaded_end = last.line_number;
                }

                self.is_fetching = false;
            }
            Err(e) => {
                eprintln!("❌ Failed to fetch lines: {}", e);
                self.is_fetching = false;
            }
        }
    }

    /// Save edited line to backend
    fn save_line(&mut self, line_number: i64) {
        let text = self.edit_buffer.clone();

        match self
            .api_client
            .update_line(&self.card_id, line_number, text.clone())
        {
            Ok(_) => {
                // Update local cache
                if let Some(line) = self
                    .loaded_lines
                    .iter_mut()
                    .find(|l| l.line_number == line_number)
                {
                    line.line_text = text;
                }
            }
            Err(e) => {
                eprintln!("❌ Failed to save line: {}", e);
            }
        }
    }

    /// Get memory usage estimate in bytes
    pub fn memory_usage(&self) -> usize {
        let lines_size: usize = self
            .loaded_lines
            .iter()
            .map(|l| l.line_text.len() + 8) // text + line_number
            .sum();

        lines_size + std::mem::size_of::<Self>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_viewer_creation() {
        let viewer = LiveNoteViewer::new("1".to_string(), "http://localhost:3000".to_string());
        assert_eq!(viewer.card_id, "1");
        assert_eq!(viewer.total_lines, 0);
        assert!(viewer.loaded_lines.is_empty());
    }

    #[test]
    fn test_memory_constraint() {
        let mut viewer = LiveNoteViewer::new("1".to_string(), "http://localhost:3000".to_string());

        // Simulate 100 lines of 100 chars each
        for i in 0..100 {
            viewer.loaded_lines.push(LineEntry {
                line_number: i,
                line_text: "x".repeat(100),
            });
        }

        let memory = viewer.memory_usage();
        assert!(memory < 30_000_000, "Memory usage {} exceeds 30 MB", memory);
    }
}
