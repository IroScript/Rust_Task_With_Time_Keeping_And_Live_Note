use egui::{Color32, Context, RichText, Ui};

use crate::virtual_scroller::LiveNoteViewer;

/// Live Note View - Virtual Scrolling Editor for massive text data
pub struct LiveNoteView {
    pub viewer: Option<LiveNoteViewer>,
    pub card_id_input: String,
    pub backend_url: String,
    pub status_message: String,
    pub is_loading: bool,
}

impl Default for LiveNoteView {
    fn default() -> Self {
        Self {
            viewer: None,
            card_id_input: String::new(),
            backend_url: "http://localhost:3000".to_string(),
            status_message: String::new(),
            is_loading: false,
        }
    }
}

impl LiveNoteView {
    pub fn new() -> Self {
        Self::default()
    }

    /// Render the live note view
    pub fn show(&mut self, ctx: &Context, ui: &mut Ui) {
        if self.viewer.is_none() {
            self.show_initialization_ui(ui);
        } else {
            self.show_viewer_ui(ctx, ui);
        }
    }

    /// Show initialization UI (before card is loaded)
    fn show_initialization_ui(&mut self, ui: &mut Ui) {
        ui.heading("📄 Live Note - Virtual Scrolling Editor");
        ui.label("Load a card to view and edit massive text data");
        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Card ID:");
            ui.text_edit_singleline(&mut self.card_id_input);
        });

        ui.horizontal(|ui| {
            ui.label("Backend URL:");
            ui.text_edit_singleline(&mut self.backend_url);
        });

        ui.add_space(10.0);

        if ui.button("🔄 Load Card").clicked() && !self.is_loading {
            self.load_card();
        }

        if self.is_loading {
            ui.spinner();
            ui.label("Loading card metadata...");
        }

        if !self.status_message.is_empty() {
            ui.add_space(10.0);
            let color = if self.status_message.starts_with("✅") {
                Color32::GREEN
            } else if self.status_message.starts_with("❌") {
                Color32::RED
            } else {
                Color32::YELLOW
            };
            ui.label(RichText::new(&self.status_message).color(color));
        }

        ui.add_space(20.0);
        ui.label("💡 Instructions:");
        ui.label("1. Enter a card ID (e.g., '1')");
        ui.label("2. Click 'Load Card' to initialize");
        ui.label("3. Use CLI tool to ingest large text files");
        ui.label("4. Scroll through billions of lines smoothly");
        ui.label("5. Click line numbers to edit individual lines");
    }

    /// Show viewer UI (after card is loaded)
    fn show_viewer_ui(&mut self, _ctx: &Context, ui: &mut Ui) {
        if let Some(ref mut viewer) = self.viewer {
            // Show stats header
            ui.horizontal(|ui| {
                ui.heading("📄 Live Note Editor");
                ui.separator();
                ui.label(format!("Card ID: {}", viewer.card_id));
            });

            ui.horizontal(|ui| {
                ui.label(format!("📊 Total Lines: {}", viewer.total_lines));
                ui.separator();
                ui.label(format!("💾 Memory: {} KB", viewer.memory_usage() / 1024));
                ui.separator();
                ui.label(format!(
                    "📦 Loaded: {}-{}",
                    viewer.loaded_start, viewer.loaded_end
                ));
            });

            ui.separator();

            // Controls
            ui.horizontal(|ui| {
                ui.label("Font Size:");
                ui.add(egui::Slider::new(&mut viewer.font_size, 8.0..=24.0));

                ui.separator();

                if ui.button("🔄 Refresh").clicked() {
                    viewer.loaded_lines.clear();
                    viewer.loaded_start = 0;
                    viewer.loaded_end = 0;
                }
            });

            // Close button outside horizontal to avoid borrowing issues
            if ui.button("❌ Close").clicked() {
                self.viewer = None;
                self.card_id_input.clear();
                self.status_message.clear();
                return;
            }

            ui.separator();

            // Render the virtual scroller
            viewer.show(ui);
        }
    }

    /// Load card from backend
    fn load_card(&mut self) {
        if self.card_id_input.is_empty() {
            self.status_message = "❌ Please enter a card ID".to_string();
            return;
        }

        self.is_loading = true;
        self.status_message = "⏳ Loading card...".to_string();

        let mut viewer = LiveNoteViewer::new(
            self.card_id_input.clone(),
            self.backend_url.clone(),
        );

        match viewer.init() {
            Ok(_) => {
                self.status_message = format!(
                    "✅ Loaded card {} with {} lines",
                    self.card_id_input, viewer.total_lines
                );
                self.viewer = Some(viewer);
                self.is_loading = false;
            }
            Err(e) => {
                self.status_message = format!("❌ Failed to load card: {}", e);
                self.is_loading = false;
            }
        }
    }

    /// Check if a card is currently loaded
    pub fn is_loaded(&self) -> bool {
        self.viewer.is_some()
    }

    /// Get current card ID
    pub fn current_card_id(&self) -> Option<&str> {
        self.viewer.as_ref().map(|v| v.card_id.as_str())
    }

    /// Unload current card
    pub fn unload(&mut self) {
        self.viewer = None;
        self.card_id_input.clear();
        self.status_message.clear();
        self.is_loading = false;
    }
}
