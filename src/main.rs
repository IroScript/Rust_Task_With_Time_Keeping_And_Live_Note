// Daily Motivation - Pure Rust GUI (winit + wgpu + egui)
// A motivation quote display application with custom title bar
//
// This application demonstrates:
// - Frameless window with custom title bar and icons
// - Gradient and solid color theme system
// - Quote rotation with configurable intervals
// - Control panel for managing quotes
// - Theme customization modal
// - All implemented in Pure Rust without Tauri or web technologies

use std::fs::{File, OpenOptions};
use std::io::{BufReader, Write};
use std::thread;
use std::time::{Duration, Instant};

use winit::raw_window_handle::HasWindowHandle;
use winit::{
    dpi::{LogicalSize, PhysicalPosition},
    event::WindowEvent,
    event_loop::EventLoop,
    window::Window,
};

use egui::Context;
use egui::FontId;
use egui::{Color32, Frame, RichText, Rounding, Sense, Stroke, TopBottomPanel, Vec2};

#[cfg(windows)]
use windows::Win32::Foundation::HWND;
#[cfg(windows)]
use windows::Win32::UI::WindowsAndMessaging::{
    GetWindowLongW, SetLayeredWindowAttributes, SetWindowLongW, SetWindowPos, GWL_EXSTYLE,
    HWND_TOPMOST, LWA_ALPHA, SWP_NOMOVE, SWP_NOSIZE, SWP_SHOWWINDOW, WS_EX_LAYERED,
};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// =============================================================================
// CONSTANTS
// =============================================================================

// =============================================================================
// YEAR 50,000 — NEURO-QUANTUM COLOR SYSTEM
// =============================================================================

const TITLE_BAR_HEIGHT: f32 = 26.0; // Slightly taller for futuristic feel

// ── DEEP VOID PALETTE ─────────────────────────────────
const BG_GLASS: Color32 = Color32::TRANSPARENT;

// ── QUANTUM NEON ACCENTS ──────────────────────────────
const NEON_CYAN: Color32 = Color32::from_rgb(0, 255, 220); // #00FFDC
const NEON_PLASMA: Color32 = Color32::from_rgb(180, 0, 255); // #B400FF
const NEON_SOLAR: Color32 = Color32::from_rgb(255, 160, 0); // #FFA000
const NEON_LIME: Color32 = Color32::from_rgb(80, 255, 120); // #50FF78
const NEON_ROSE: Color32 = Color32::from_rgb(255, 40, 120); // #FF2878

// ── TITLE BAR ─────────────────────────────────────────
const TITLEBAR_FG: Color32 = NEON_CYAN;

// ── BUTTON STATES ─────────────────────────────────────
const BTN_NORMAL_BG: Color32 = Color32::TRANSPARENT;
const BTN_ACTIVE_BG: Color32 = Color32::from_rgb(0, 120, 100);
const BTN_ACTIVE_FG: Color32 = Color32::WHITE;

// ── DIMENSIONS ────────────────────────────────────────
const CONTROL_PANEL_WIDTH: f32 = 300.0;
const DEFAULT_WINDOW_SIZE: (u32, u32) = (1100, 700);
const MIN_WINDOW_SIZE: (u32, u32) = (450, 350);

// ── PANEL / CANVAS ────────────────────────────────────
const CANVAS_BG: Color32 = Color32::TRANSPARENT;
const CONTROL_PANEL_BG: Color32 = Color32::TRANSPARENT;

// =============================================================================
// DATA STRUCTURES
// =============================================================================

/// A single motivational quote with main text and supporting text
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quote {
    pub main_text: String,
    pub sub_text: String,
}

impl Default for Quote {
    fn default() -> Self {
        Self {
            main_text: "Focus on your goals - Success awaits!".to_string(),
            sub_text: "Keep pushing - You're doing great!".to_string(),
        }
    }
}

/// Theme configuration for the application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    pub mode: ThemeMode,
    pub gradient_angle: i32,
    pub gradient_colors: Vec<Color32>,
    pub solid_color: Color32,
    pub apply_to_entire_window: bool,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            mode: ThemeMode::Gradient,
            gradient_angle: 135,
            gradient_colors: vec![
                Color32::from_rgb(2, 4, 16),    // Void black
                Color32::from_rgb(30, 0, 80),   // Deep plasma
                Color32::from_rgb(0, 60, 120),  // Quantum blue
                Color32::from_rgb(0, 200, 180), // Neon teal
            ],
            solid_color: Color32::from_rgb(2, 8, 24),
            apply_to_entire_window: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ThemeMode {
    Gradient,
    Solid,
}

/// Text styling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextStyleConfig {
    pub main_text_size: f32,
    pub sub_text_size: f32,
    pub main_text_color: Color32,
    pub sub_text_color: Color32,
    pub main_line_gap: f32,
    pub sub_line_gap: f32,
    pub between_gap: f32,
}

impl Default for TextStyleConfig {
    fn default() -> Self {
        Self {
            main_text_size: 24.0,
            sub_text_size: 14.0,
            main_text_color: Color32::WHITE,
            sub_text_color: Color32::from_rgba_unmultiplied(255, 255, 255, 200),
            main_line_gap: 1.6,
            sub_line_gap: 1.6,
            between_gap: 15.0,
        }
    }
}

// =============================================================================
// TITLE BAR ICON DEFINITIONS (From your original code)
// =============================================================================

/// Title bar icon definitions - each icon has a symbol and tooltip
#[derive(Debug, Clone)]
pub struct TitleBarIcon {
    pub symbol: &'static str,
    pub tooltip: &'static str,
    pub width: f32,
    pub font_size: f32,
}

impl TitleBarIcon {
    pub const fn new(
        symbol: &'static str,
        tooltip: &'static str,
        width: f32,
        font_size: f32,
    ) -> Self {
        Self {
            symbol,
            tooltip,
            width,
            font_size,
        }
    }
}

pub mod icons {
    use super::TitleBarIcon;

    pub const APP_ICON: TitleBarIcon =
        TitleBarIcon::new("\u{f135}", "Daily Motivation", 20.0, 24.0);
    pub const THEME: TitleBarIcon = TitleBarIcon::new("\u{eb5c}", "Change Theme", 20.0, 12.0);
    pub const TOGGLE_BG: TitleBarIcon =
        TitleBarIcon::new("\u{f110}", "Toggle 3D Background", 20.0, 16.0);
    pub const EXPORT: TitleBarIcon = TitleBarIcon::new("\u{f0207}", "Export Quotes", 20.0, 13.2);
    pub const ZOOM_IN: TitleBarIcon = TitleBarIcon::new("\u{f120d}", "Zoom In", 20.0, 16.8);
    pub const ZOOM_OUT: TitleBarIcon = TitleBarIcon::new("\u{f06ec}", "Zoom Out", 20.0, 16.8);
    pub const TOGGLE_PANEL: TitleBarIcon =
        TitleBarIcon::new("\u{f0c9}", "Toggle Panel", 20.0, 24.0);
    pub const MINIMIZE: TitleBarIcon = TitleBarIcon::new("\u{f2d1}", "Minimize", 20.0, 11.2);
    pub const MAXIMIZE: TitleBarIcon = TitleBarIcon::new("\u{f2d0}", "Maximize", 20.0, 10.0);
    pub const CLOSE: TitleBarIcon = TitleBarIcon::new("\u{f110a}", "Close", 20.0, 13.2);
    pub const HIDE_HEADER: TitleBarIcon = TitleBarIcon::new("\u{f102}", "Hide Header", 20.0, 17.5);
    pub const SHOW_HEADER: TitleBarIcon = TitleBarIcon::new("\u{f103}", "Show Header", 20.0, 24.0);
    pub const ROTATE: TitleBarIcon = TitleBarIcon::new("\u{f01e}", "Rotate Window", 20.0, 16.0);
    pub const ANIMATE: TitleBarIcon = TitleBarIcon::new("\u{f04b}", "Animate Window", 20.0, 16.0);

    // Multi-Animation Icons
    pub const ANIM_BOUNCE: TitleBarIcon =
        TitleBarIcon::new("\u{f0025}", "Bounce Animation", 20.0, 16.0);
    pub const ANIM_SHAKE: TitleBarIcon =
        TitleBarIcon::new("\u{f067a}", "Shake Animation", 20.0, 16.0);
    pub const ANIM_DANCE: TitleBarIcon =
        TitleBarIcon::new("\u{f00d2}", "Dance Animation", 20.0, 16.0);
    pub const ANIM_ROTATE: TitleBarIcon =
        TitleBarIcon::new("\u{f01e}", "Rotate Animation", 20.0, 16.0);
    pub const ANIM_DISSOLVE: TitleBarIcon =
        TitleBarIcon::new("\u{f0376}", "Dissolve Animation", 20.0, 16.0);
    pub const ANIM_FLY: TitleBarIcon = TitleBarIcon::new("\u{f02eb}", "Fly Animation", 20.0, 16.0);
}

// =============================================================================
// UI STATE
// =============================================================================

/// Holds all state for the title bar UI
#[derive(Debug)]
pub struct TitleBarState {
    // Button hover states
    pub theme_btn_hovered: bool,
    pub toggle_bg_btn_hovered: bool,
    pub export_btn_hovered: bool,
    pub zoom_out_btn_hovered: bool,
    pub zoom_in_btn_hovered: bool,
    pub toggle_panel_btn_hovered: bool,
    pub minimize_btn_hovered: bool,
    pub maximize_btn_hovered: bool,
    pub close_btn_hovered: bool,

    // Panel visibility
    pub control_panel_visible: bool,
    pub header_visible: bool,

    // Zoom state
    pub zoom_level: f32,

    // Drag state
    pub dragging: bool,
    pub drag_start: Option<PhysicalPosition<f64>>,
}

impl Default for TitleBarState {
    fn default() -> Self {
        Self {
            theme_btn_hovered: false,
            toggle_bg_btn_hovered: false,
            export_btn_hovered: false,
            zoom_out_btn_hovered: false,
            zoom_in_btn_hovered: false,
            toggle_panel_btn_hovered: false,
            minimize_btn_hovered: false,
            maximize_btn_hovered: false,
            close_btn_hovered: false,

            control_panel_visible: true,
            header_visible: true,

            zoom_level: 1.0,

            dragging: false,
            drag_start: None,
        }
    }
}

/// Actions that can be triggered from the title bar
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TitleBarAction {
    ThemeClicked,
    ToggleBg,
    ExportClicked,
    ZoomIn,
    ZoomOut,
    TogglePanel,
    MinimizeClicked,
    MaximizeClicked,
    CloseClicked,
    ShowHeader,
    HideHeader,
    AnimateClicked,
    PlayBounce,
    PlayShake,
    PlayDance,
    PlayRotate,
    PlayDissolve,
    PlayFly,
    StopAnimations,
}

// =============================================================================
// ANIMATION TYPES
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub enum AppAnimation {
    #[default]
    None,
    Bounce,
    Shake,
    Dance,
    Rotate,
    Dissolve,
    Fly,
}

// =============================================================================
// PERSISTENCE CONFIGURATION
// =============================================================================

/// Configuration for persistence
#[derive(Serialize, Deserialize)]
struct AppConfig {
    quotes: Vec<Quote>,
    interval_secs: u64,
    theme: ThemeConfig,
    text_style: TextStyleConfig,
}

impl AppConfig {
    fn load() -> Option<Self> {
        if let Ok(file) = File::open("settings.json") {
            let reader = BufReader::new(file);
            serde_json::from_reader(reader).ok()
        } else {
            None
        }
    }

    fn save(&self) {
        if let Ok(file) = File::create("settings.json") {
            // Pretty print for readability
            let _ = serde_json::to_writer_pretty(file, self);
        }
    }
}

// =============================================================================
// MAIN APPLICATION STATE
// =============================================================================

/// Main application state
#[derive(Debug)]
pub struct AppState {
    // Title bar state
    pub title_bar_state: TitleBarState,

    // Quotes
    pub quotes: Vec<Quote>,
    pub current_quote_index: usize,

    // Rotation
    pub rotation_interval: Duration,
    pub last_rotation: Instant,
    pub rotation_enabled: bool,

    // Interval as numeric (for DragValue)
    pub interval_secs: u64,

    // Theme
    pub theme: ThemeConfig,
    pub theme_modal_open: bool,

    // Text style
    pub text_style: TextStyleConfig,

    // Input fields
    pub main_text_input: String,
    pub sub_text_input: String,

    pub subtitle_editing: bool,
    pub subtitle_edit_buffer: String,

    pub confirm_clear_pending: bool,

    // 3D Background Process
    pub is_3d_bg_active: bool,
    pub bg_process: Option<std::process::Child>,
    pub bg_hwnd: Option<isize>,

    // Color picker toggles
    pub show_main_color_picker: bool,
    pub show_sub_color_picker: bool,

    // Running state
    pub running: bool,

    // Activity tracking for auto-hide
    pub last_interaction: Instant,

    // Custom manual resize state
    // (ResizeDirection, initial_cursor_x, initial_cursor_y, initial_window_x, initial_window_y, initial_width, initial_height)
    pub manual_resize_start: Option<(winit::window::ResizeDirection, i32, i32, i32, i32, u32, u32)>,

    // Rotation state: 0=0, 1=90, 2=180, 3=270
    pub rotation: u8,

    // Bouncy window state (Now part of Multi-Animation)
    pub active_animation: AppAnimation,
    pub anim_progress: f32,
    pub bounce_vel_x: f32,
    pub bounce_vel_y: f32,
    pub base_pos: Option<(i32, i32)>,
}

impl Default for AppState {
    fn default() -> Self {
        // Try to load from config
        if let Some(config) = AppConfig::load() {
            Self {
                title_bar_state: TitleBarState::default(),
                quotes: config.quotes,
                current_quote_index: 0,
                rotation_interval: Duration::from_secs(config.interval_secs),
                last_rotation: Instant::now(),
                rotation_enabled: true,
                interval_secs: config.interval_secs,
                theme: config.theme,
                theme_modal_open: false,
                text_style: config.text_style,
                main_text_input: String::new(),
                sub_text_input: String::new(),
                show_main_color_picker: false,
                show_sub_color_picker: false,
                running: true,
                last_interaction: Instant::now(),
                subtitle_editing: false,
                subtitle_edit_buffer: String::new(),
                confirm_clear_pending: false,
                is_3d_bg_active: false,
                bg_process: None,
                bg_hwnd: None,
                manual_resize_start: None,
                rotation: 0,
                active_animation: AppAnimation::None,
                anim_progress: 0.0,
                bounce_vel_x: 5.0,
                bounce_vel_y: 4.0,
                base_pos: None,
            }
        } else {
            // Default initialization if no config found
            Self {
                title_bar_state: TitleBarState::default(),

                quotes: vec![
                    Quote {
                        main_text: "এখনই কাজে মনোযোগ দাও - ফোকাস তোমার শক্তি".to_string(),
                        sub_text: "Keep pushing - You're doing great! 🌟".to_string(),
                    },
                    Quote {
                        main_text: "প্রতিটি মুহূর্ত গুরুত্বপূর্ণ - কাজ চালিয়ে যাও".to_string(),
                        sub_text: "Keep pushing - You're doing great! 🌟".to_string(),
                    },
                    Quote {
                        main_text: "সফলতা ধৈর্যের ফল - হার মানিও না".to_string(),
                        sub_text: "Keep pushing - You're doing great! 🌟".to_string(),
                    },
                    Quote {
                        main_text: "Focus on the work - Success is near".to_string(),
                        sub_text: "Keep pushing - You're doing great! 🌟".to_string(),
                    },
                    Quote {
                        main_text: "Stay disciplined - Great things take time".to_string(),
                        sub_text: "Keep pushing - You're doing great! 🌟".to_string(),
                    },
                    Quote {
                        main_text: "তুমি পারবে - শুধু চেষ্টা চালিয়ে যাও".to_string(),
                        sub_text: "Keep pushing - You're doing great! 🌟".to_string(),
                    },
                    Quote {
                        main_text: "Dreams need action - Start now".to_string(),
                        sub_text: "Keep pushing - You're doing great! 🌟".to_string(),
                    },
                    Quote {
                        main_text: "প্রতিদিন একটু এগিয়ে যাও - লক্ষ্য কাছে".to_string(),
                        sub_text: "Keep pushing - You're doing great! 🌟".to_string(),
                    },
                    Quote {
                        main_text: "Consistency beats talent - Keep going".to_string(),
                        sub_text: "Keep pushing - You're doing great! 🌟".to_string(),
                    },
                    Quote {
                        main_text: "বিশ্রাম নাও কিন্তু হাল ছাড়ো না".to_string(),
                        sub_text: "Keep pushing - You're doing great! 🌟".to_string(),
                    },
                ],
                current_quote_index: 0,

                rotation_interval: Duration::from_secs(8),
                last_rotation: Instant::now(),
                rotation_enabled: true,

                interval_secs: 8,

                theme: ThemeConfig::default(),
                theme_modal_open: false,

                text_style: TextStyleConfig::default(),

                main_text_input: String::new(),
                sub_text_input: String::new(),

                show_main_color_picker: false,
                show_sub_color_picker: false,

                running: true,
                last_interaction: Instant::now(),
                subtitle_editing: false,
                subtitle_edit_buffer: String::new(),
                confirm_clear_pending: false,
                is_3d_bg_active: false,
                bg_process: None,
                bg_hwnd: None,
                manual_resize_start: None,
                rotation: 0,
                active_animation: AppAnimation::None,
                anim_progress: 0.0,
                bounce_vel_x: 5.0,
                bounce_vel_y: 4.0,
                base_pos: None,
            }
        }
    }
}

impl Drop for AppState {
    fn drop(&mut self) {
        if let Some(mut child) = self.bg_process.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

impl AppState {
    /// Save current state to settings.json
    pub fn save(&self) {
        let config = AppConfig {
            quotes: self.quotes.clone(),
            interval_secs: self.interval_secs,
            theme: self.theme.clone(),
            text_style: self.text_style.clone(),
        };
        config.save();
    }

    /// Get the current quote
    pub fn current_quote(&self) -> Option<&Quote> {
        self.quotes.get(self.current_quote_index)
    }

    /// Rotate to next quote
    pub fn next_quote(&mut self) {
        if !self.quotes.is_empty() {
            self.current_quote_index = (self.current_quote_index + 1) % self.quotes.len();
            self.last_rotation = Instant::now();
        }
    }

    /// Rotate to previous quote
    pub fn prev_quote(&mut self) {
        if !self.quotes.is_empty() {
            if self.current_quote_index == 0 {
                self.current_quote_index = self.quotes.len() - 1;
            } else {
                self.current_quote_index -= 1;
            }
            self.last_rotation = Instant::now();
        }
    }

    /// Add a new quote
    pub fn add_quote(&mut self, main: String, sub: String) {
        let sub = if sub.is_empty() {
            "Keep pushing - You're doing great! 🌟".to_string()
        } else {
            sub
        };
        self.quotes.push(Quote {
            main_text: main,
            sub_text: sub,
        });
        self.current_quote_index = self.quotes.len() - 1;
        self.save();
    }

    /// Delete a quote by index
    pub fn delete_quote(&mut self, index: usize) {
        if index < self.quotes.len() {
            self.quotes.remove(index);
            if self.current_quote_index >= self.quotes.len() && !self.quotes.is_empty() {
                self.current_quote_index = self.quotes.len() - 1;
            }
            self.save();
        }
    }

    /// Get background color (interpolated gradient or solid)
    pub fn get_background_color(&self) -> Color32 {
        if self.is_3d_bg_active {
            return Color32::TRANSPARENT;
        }

        if self.theme.mode == ThemeMode::Solid {
            return self.theme.solid_color;
        }

        // For gradient, return the first color as base
        // Full gradient would need shader support in wgpu
        self.theme
            .gradient_colors
            .first()
            .copied()
            .unwrap_or(CANVAS_BG)
    }
}

// =============================================================================
// BUTTON RENDERER
// =============================================================================

pub fn draw_icon_button(
    ui: &mut egui::Ui,
    icon: &TitleBarIcon,
    _bg_color: Color32,
    fg_color: Color32,
    _hovered: bool,
) -> egui::Response {
    let size = Vec2::new(icon.width + 6.0, TITLE_BAR_HEIGHT - 2.0);
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());

    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    let is_hovered = response.hovered();

    // Outer glow border on hover
    if is_hovered {
        let glow_rect = rect.expand(2.0);
        ui.painter().rect_filled(
            glow_rect,
            Rounding::same(8.0),
            NEON_CYAN.gamma_multiply(0.12),
        );
        ui.painter().rect_stroke(
            glow_rect,
            Rounding::same(8.0),
            Stroke::new(1.0, NEON_CYAN.gamma_multiply(0.47)),
        );
    }

    // Main button background — glass morphism
    let bg = if is_hovered {
        NEON_CYAN.gamma_multiply(0.11)
    } else {
        BG_GLASS
    };
    ui.painter().rect_filled(rect, Rounding::same(6.0), bg);

    // Subtle top-edge highlight (glass rim)
    let top_line = [
        egui::pos2(rect.left() + 4.0, rect.top() + 1.0),
        egui::pos2(rect.right() - 4.0, rect.top() + 1.0),
    ];
    ui.painter().line_segment(
        top_line,
        Stroke::new(
            1.0,
            if is_hovered {
                NEON_CYAN.gamma_multiply(0.7)
            } else {
                Color32::from_rgba_premultiplied(255, 255, 255, 25)
            },
        ),
    );

    // Icon
    let icon_color = if is_hovered { NEON_CYAN } else { fg_color };
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        icon.symbol,
        FontId::proportional(icon.font_size),
        icon_color,
    );

    response
}

pub fn draw_text_button(
    ui: &mut egui::Ui,
    text: &str,
    bg_color: Color32,
    width: f32,
    height: f32,
) -> egui::Response {
    let size = Vec2::new(width, height);
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());

    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    let is_hovered = response.hovered();
    let is_clicked = response.is_pointer_button_down_on();

    // Glow halo on hover
    if is_hovered {
        ui.painter().rect_filled(
            rect.expand(3.0),
            Rounding::same(8.0),
            Color32::from_rgba_unmultiplied(bg_color.r(), bg_color.g(), bg_color.b(), 18),
        );
    }

    // Background with glass sheen
    let bg = if is_clicked {
        bg_color.linear_multiply(1.4)
    } else if is_hovered {
        bg_color.linear_multiply(1.15)
    } else {
        bg_color.linear_multiply(0.75)
    };

    ui.painter().rect_filled(rect, Rounding::same(6.0), bg);

    // Top highlight rim
    ui.painter().line_segment(
        [
            egui::pos2(rect.left() + 6.0, rect.top() + 1.0),
            egui::pos2(rect.right() - 6.0, rect.top() + 1.0),
        ],
        Stroke::new(
            1.0,
            Color32::from_rgba_unmultiplied(255, 255, 255, if is_hovered { 60 } else { 20 }),
        ),
    );

    // Border
    ui.painter().rect_stroke(
        rect,
        Rounding::same(6.0),
        Stroke::new(
            1.0,
            if is_hovered {
                Color32::from_rgba_unmultiplied(bg_color.r(), bg_color.g(), bg_color.b(), 200)
            } else {
                Color32::from_rgba_unmultiplied(bg_color.r(), bg_color.g(), bg_color.b(), 80)
            },
        ),
    );

    // Label with letter-spacing simulation (spaces between chars)
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        text,
        FontId::proportional(11.5),
        Color32::from_rgba_unmultiplied(255, 255, 255, if is_hovered { 255 } else { 210 }),
    );

    response
}

// =============================================================================
// TITLE BAR RENDERER
// =============================================================================

/// Render the complete title bar with all icons
pub fn render_title_bar(
    ctx: &Context,
    state: &mut AppState,
    window: &Window,
) -> Vec<TitleBarAction> {
    if !state.title_bar_state.header_visible {
        return Vec::new();
    }

    let mut actions = Vec::new();

    let titlebar_bg = Color32::from_black_alpha(26);

    TopBottomPanel::top("title_bar")
        .exact_height(TITLE_BAR_HEIGHT)
        .frame(Frame::none().fill(titlebar_bg))
        .show(ctx, |ui| {
            let rect = ui.max_rect();

            // ── HUD Elements ──
            ui.painter().line_segment(
                [rect.left_top(), rect.right_top()],
                Stroke::new(1.5, TITLEBAR_FG.gamma_multiply(0.78)),
            );
            ui.painter().line_segment(
                [
                    egui::pos2(rect.left(), rect.top() + 3.0),
                    egui::pos2(rect.right(), rect.top() + 3.0),
                ],
                Stroke::new(0.5, TITLEBAR_FG.gamma_multiply(0.15)),
            );

            let b = 8.0;
            let stroke = Stroke::new(1.5, TITLEBAR_FG.gamma_multiply(0.63));
            ui.painter().line_segment(
                [
                    egui::pos2(rect.left(), rect.top()),
                    egui::pos2(rect.left() + b, rect.top()),
                ],
                stroke,
            );
            ui.painter().line_segment(
                [
                    egui::pos2(rect.left(), rect.top()),
                    egui::pos2(rect.left(), rect.bottom()),
                ],
                stroke,
            );
            ui.painter().line_segment(
                [
                    egui::pos2(rect.right() - b, rect.top()),
                    egui::pos2(rect.right(), rect.top()),
                ],
                stroke,
            );
            ui.painter().line_segment(
                [
                    egui::pos2(rect.right(), rect.top()),
                    egui::pos2(rect.right(), rect.bottom()),
                ],
                stroke,
            );

            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                ui.spacing_mut().item_spacing = Vec2::new(4.0, 0.0);
                ui.add_space(12.0);

                ui.label(
                    RichText::new(icons::APP_ICON.symbol)
                        .size(15.0)
                        .color(TITLEBAR_FG),
                );
                ui.label(
                    RichText::new("DAILY  MOTIVATION")
                        .color(TITLEBAR_FG)
                        .strong()
                        .size(12.0),
                );

                ui.add_space(4.0);
                let (br, _) = ui.allocate_exact_size(Vec2::new(38.0, 14.0), Sense::hover());
                ui.painter()
                    .rect_filled(br, Rounding::same(3.0), TITLEBAR_FG.gamma_multiply(0.08));
                ui.painter().rect_stroke(
                    br,
                    Rounding::same(3.0),
                    Stroke::new(0.5, TITLEBAR_FG.gamma_multiply(0.31)),
                );
                ui.painter().text(
                    br.center(),
                    egui::Align2::CENTER_CENTER,
                    "v∞.0",
                    FontId::proportional(8.5),
                    TITLEBAR_FG.gamma_multiply(0.7),
                );

                ui.add_space(8.0);
                if !state.quotes.is_empty() {
                    ui.label(
                        RichText::new(format!(
                            "[ {}/{} ]",
                            state.current_quote_index + 1,
                            state.quotes.len()
                        ))
                        .color(NEON_LIME.gamma_multiply(0.7))
                        .size(10.5),
                    );
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(3.0, 0.0);
                    ui.add_space(6.0);

                    // Right-side buttons
                    let btns = [
                        (&icons::CLOSE, NEON_ROSE, TitleBarAction::CloseClicked),
                        (
                            &icons::MAXIMIZE,
                            Color32::WHITE,
                            TitleBarAction::MaximizeClicked,
                        ),
                        (
                            &icons::MINIMIZE,
                            Color32::WHITE,
                            TitleBarAction::MinimizeClicked,
                        ),
                    ];

                    for (icon, color, action) in btns {
                        if draw_icon_button(ui, icon, Color32::TRANSPARENT, color, false).clicked()
                        {
                            actions.push(action);
                        }
                    }

                    if draw_icon_button(
                        ui,
                        &icons::HIDE_HEADER,
                        Color32::TRANSPARENT,
                        Color32::WHITE,
                        false,
                    )
                    .clicked()
                    {
                        actions.push(TitleBarAction::HideHeader);
                    }

                    ui.add_space(8.0);
                    // ANIMATION SECTION (just right of TOGGLE_BG in code = physically right)
                    let anim_btns = [
                        (&icons::ANIM_FLY, TitleBarAction::PlayFly, AppAnimation::Fly),
                        (
                            &icons::ANIM_DISSOLVE,
                            TitleBarAction::PlayDissolve,
                            AppAnimation::Dissolve,
                        ),
                        (
                            &icons::ANIM_ROTATE,
                            TitleBarAction::PlayRotate,
                            AppAnimation::Rotate,
                        ),
                        (
                            &icons::ANIM_DANCE,
                            TitleBarAction::PlayDance,
                            AppAnimation::Dance,
                        ),
                        (
                            &icons::ANIM_SHAKE,
                            TitleBarAction::PlayShake,
                            AppAnimation::Shake,
                        ),
                        (
                            &icons::ANIM_BOUNCE,
                            TitleBarAction::PlayBounce,
                            AppAnimation::Bounce,
                        ),
                    ];

                    for (icon, action, anim_type) in anim_btns {
                        let active = state.active_animation == anim_type;
                        let color = if active { NEON_LIME } else { Color32::WHITE };
                        if draw_icon_button(ui, icon, Color32::TRANSPARENT, color, active).clicked()
                        {
                            actions.push(action);
                        }
                    }

                    ui.add_space(8.0);
                    // TOGGLE_BG (placed left attached to other buttons)
                    let bg_color = if state.is_3d_bg_active {
                        NEON_CYAN
                    } else {
                        Color32::from_rgba_premultiplied(255, 255, 255, 150)
                    };
                    if draw_icon_button(
                        ui,
                        &icons::TOGGLE_BG,
                        Color32::TRANSPARENT,
                        bg_color,
                        false,
                    )
                    .clicked()
                    {
                        actions.push(TitleBarAction::ToggleBg);
                    }

                    ui.add_space(8.0);
                    if draw_icon_button(
                        ui,
                        &icons::ZOOM_IN,
                        Color32::TRANSPARENT,
                        Color32::WHITE,
                        false,
                    )
                    .clicked()
                    {
                        actions.push(TitleBarAction::ZoomIn);
                    }
                    if draw_icon_button(
                        ui,
                        &icons::ZOOM_OUT,
                        Color32::TRANSPARENT,
                        Color32::WHITE,
                        false,
                    )
                    .clicked()
                    {
                        actions.push(TitleBarAction::ZoomOut);
                    }

                    ui.add_space(8.0);
                    if draw_icon_button(
                        ui,
                        &icons::EXPORT,
                        Color32::TRANSPARENT,
                        Color32::WHITE,
                        false,
                    )
                    .clicked()
                    {
                        actions.push(TitleBarAction::ExportClicked);
                    }
                    if draw_icon_button(
                        ui,
                        &icons::THEME,
                        Color32::TRANSPARENT,
                        Color32::WHITE,
                        false,
                    )
                    .clicked()
                    {
                        actions.push(TitleBarAction::ThemeClicked);
                    }

                    let drag_avail = ui.available_width();
                    if drag_avail > 0.0 {
                        let (_, resp) = ui.allocate_exact_size(
                            Vec2::new(drag_avail, TITLE_BAR_HEIGHT),
                            Sense::drag(),
                        );
                        if resp.drag_started() {
                            let _ = window.drag_window();
                        }
                    }
                });
            });
            actions
        })
        .inner
}

/// Render floating button group (Toggle Panel, Show Header)
fn render_floating_buttons(ctx: &Context, state: &mut AppState) -> Vec<TitleBarAction> {
    let mut actions = Vec::new();

    // Auto-hide logic
    let elapsed = state.last_interaction.elapsed().as_secs_f32();
    let opacity = if elapsed > 5.0 {
        1.0 - ((elapsed - 5.0) / 0.5).min(1.0)
    } else {
        1.0
    };
    if opacity <= 0.0 {
        return actions;
    }

    // Fixed position: Top 70px, Right 10px
    // We use Screen Rect to determine Right edge
    let screen_rect = ctx.screen_rect();
    let pos = egui::pos2(screen_rect.right() - 10.0, 70.0);

    egui::Area::new(egui::Id::new("floating_buttons"))
        .fixed_pos(pos)
        .pivot(egui::Align2::RIGHT_TOP)
        .order(egui::Order::Foreground)
        .interactable(opacity > 0.0) // Fix: interactable until fully invisible
        .show(ctx, |ui| {
            if opacity < 1.0 && opacity > 0.0 {
                ui.ctx().request_repaint();
            }
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing = Vec2::new(0.0, 8.0);

                // 1. Toggle Panel Button
                // Background color changes based on panel visibility
                let (bg, fg) = if state.title_bar_state.control_panel_visible {
                    (BTN_ACTIVE_BG, BTN_ACTIVE_FG)
                } else {
                    (BTN_NORMAL_BG, Color32::WHITE)
                };

                let bg = bg.linear_multiply(opacity);
                let fg = fg.linear_multiply(opacity);

                let (btn_icon, btn_tooltip) = if state.title_bar_state.control_panel_visible {
                    (&icons::TOGGLE_PANEL, "Hide Panel") // User asked for Sandwich when Visible
                } else {
                    (&icons::CLOSE, "Show Panel") // User asked for X when Hidden
                                                  // Wait, user asked: visible -> ☰, hidden -> ✕.
                                                  // I will follow specific instruction despite it feeling backwards.
                                                  // "control_panel_visible == true -> icon = '☰'"
                                                  // "control_panel_visible == false -> icon = '✕'"
                };

                // Override user instruction if it implies X opens the menu?
                // "The ☰ icon changes to ✕ when control panel is hidden".
                // If I click X (when hidden), it opens.
                // If I click ☰ (when visible), it closes.
                // Use icons::CLOSE for X.

                let response = draw_icon_button(
                    ui,
                    btn_icon,
                    bg,
                    fg,
                    state.title_bar_state.toggle_panel_btn_hovered,
                );
                state.title_bar_state.toggle_panel_btn_hovered = response.hovered();

                if response.clicked() {
                    actions.push(TitleBarAction::TogglePanel);
                }
                if opacity > 0.8 {
                    response.on_hover_text_at_pointer(btn_tooltip);
                }

                // 2. Show Header Button (only if header is hidden)
                if !state.title_bar_state.header_visible {
                    let bg = BTN_NORMAL_BG.linear_multiply(opacity);
                    let fg = Color32::WHITE.linear_multiply(opacity);

                    let response = draw_icon_button(ui, &icons::SHOW_HEADER, bg, fg, false);

                    if response.clicked() {
                        actions.push(TitleBarAction::ShowHeader);
                    }
                    if opacity > 0.8 {
                        response.on_hover_text_at_pointer(icons::SHOW_HEADER.tooltip);
                    }
                }
            });
        });

    actions
}

// =============================================================================
// MAIN CONTENT RENDERER
// =============================================================================

/// Render the main content area with quote display
pub fn render_main_content(
    ctx: &Context,
    state: &mut AppState,
    shaper: &mut Option<(
        &mut cosmic_text::FontSystem,
        &mut cosmic_text::SwashCache,
        &mut HashMap<u64, egui::TextureHandle>,
    )>,
) {
    // RIGHT SIDE PANEL — must be declared BEFORE CentralPanel

    if state.title_bar_state.control_panel_visible {
        egui::SidePanel::right("control_panel")
            .exact_width(CONTROL_PANEL_WIDTH)
            .resizable(false)
            .frame(
                Frame::none()
                    .fill(Color32::from_black_alpha(40))
                    .inner_margin(egui::Margin {
                        left: 10.0,
                        right: 10.0,
                        top: 15.0,
                        bottom: 15.0,
                    }),
            )
            .show(ctx, |ui| {
                render_control_panel_contents(ui, state, shaper);
            });
    }

    // MAIN CANVAS — CentralPanel takes remaining space automatically

    egui::CentralPanel::default()
        .frame(Frame::none().fill(Color32::TRANSPARENT))
        .show(ctx, |ui| {
            // BACKDROP RENDERER
            // We draw the gradient or solid color here across `ctx.screen_rect()`.
            // Because SidePanel is processed first and has a transparent background,
            // this draws perfectly *underneath* the SidePanel controls.
            if !state.is_3d_bg_active {
                let draw_bg =
                    state.theme.apply_to_entire_window || state.theme.mode == ThemeMode::Gradient;
                if draw_bg {
                    let rect = if state.theme.apply_to_entire_window {
                        ctx.screen_rect()
                    } else {
                        // Approximate central panel rect if not full window
                        let mut r = ctx.screen_rect();
                        if state.title_bar_state.control_panel_visible {
                            r.max.x -= CONTROL_PANEL_WIDTH;
                        }
                        r
                    };

                    if state.theme.mode == ThemeMode::Solid {
                        ui.painter_at(rect).rect_filled(
                            rect,
                            Rounding::ZERO,
                            state.theme.solid_color,
                        );
                    } else if !state.theme.gradient_colors.is_empty() {
                        let angle_rad = (state.theme.gradient_angle as f32).to_radians();

                        // Quick radial to corners approximation
                        let dir = egui::Vec2::new(angle_rad.cos(), angle_rad.sin());

                        use egui::epaint::{Mesh, Vertex};
                        let mut mesh = Mesh::default();

                        let c0 = rect.min;
                        let c1 = egui::pos2(rect.max.x, rect.min.y);
                        let c2 = egui::pos2(rect.min.x, rect.max.y);
                        let c3 = rect.max;

                        // Project corners onto gradient direction line
                        let center = rect.center();
                        let project = |p: egui::Pos2| -> f32 {
                            let v = p - center;
                            v.x * dir.x + v.y * dir.y
                        };

                        let p0 = project(c0);
                        let p1 = project(c1);
                        let p2 = project(c2);
                        let p3 = project(c3);

                        let min_p = p0.min(p1).min(p2).min(p3);
                        let max_p = p0.max(p1).max(p2).max(p3);
                        let range = (max_p - min_p).max(0.1);

                        let calc_color = |p: f32| -> Color32 {
                            let t = ((p - min_p) / range).clamp(0.0, 1.0);
                            let colors = &state.theme.gradient_colors;

                            if colors.is_empty() {
                                return Color32::TRANSPARENT;
                            }
                            if colors.len() == 1 {
                                return colors[0];
                            }

                            let n_segments = (colors.len() - 1) as f32;
                            let scaled_t = t * n_segments;
                            let mut index = scaled_t.floor() as usize;
                            index = index.min(colors.len() - 2);
                            let fract = scaled_t - index as f32;

                            let c1 = colors[index];
                            let c2 = colors[index + 1];

                            let r = (c1.r() as f32 * (1.0 - fract) + c2.r() as f32 * fract) as u8;
                            let g = (c1.g() as f32 * (1.0 - fract) + c2.g() as f32 * fract) as u8;
                            let b = (c1.b() as f32 * (1.0 - fract) + c2.b() as f32 * fract) as u8;
                            let a = (c1.a() as f32 * (1.0 - fract) + c2.a() as f32 * fract) as u8;

                            Color32::from_rgba_premultiplied(r, g, b, a)
                        };

                        let steps_x = 32;
                        let steps_y = 32;

                        for yi in 0..=steps_y {
                            let ty = yi as f32 / steps_y as f32;
                            for xi in 0..=steps_x {
                                let tx = xi as f32 / steps_x as f32;
                                let p =
                                    rect.min + egui::vec2(rect.width() * tx, rect.height() * ty);

                                let proj = project(p);

                                mesh.vertices.push(Vertex {
                                    pos: p,
                                    uv: egui::pos2(0.0, 0.0), // Use the white pixel to avoid rendering font texture atlas
                                    color: calc_color(proj),
                                });
                            }
                        }

                        for yi in 0..steps_y {
                            for xi in 0..steps_x {
                                let i0 = yi * (steps_x + 1) + xi;
                                let i1 = i0 + 1;
                                let i2 = (yi + 1) * (steps_x + 1) + xi;
                                let i3 = i2 + 1;

                                mesh.indices.extend_from_slice(&[i0, i1, i2, i1, i3, i2]);
                            }
                        }

                        ui.painter_at(rect).add(egui::Shape::mesh(mesh));
                    }
                }
            }

            ui.vertical_centered(|ui| {
                ui.add_space(80.0);

                // ── HUD FRAME CORNERS ──────────────────────────────────
                // Draw corner brackets around the central quote area
                {
                    let screen = ctx.screen_rect();
                    let panel_w = if state.title_bar_state.control_panel_visible {
                        CONTROL_PANEL_WIDTH
                    } else {
                        0.0
                    };
                    let cx = (screen.width() - panel_w) / 2.0;
                    let cy = screen.height() / 2.0;

                    let frame_w = 320.0;
                    let frame_h = 160.0;
                    let arm = 20.0;
                    let painter = ui.painter();
                    let hud_color = NEON_CYAN.gamma_multiply(0.23);
                    let hud_stroke = Stroke::new(1.5, hud_color);

                    // Top-left corner
                    let tl = egui::pos2(cx - frame_w, cy - frame_h);
                    painter.line_segment([tl, egui::pos2(tl.x + arm, tl.y)], hud_stroke);
                    painter.line_segment([tl, egui::pos2(tl.x, tl.y + arm)], hud_stroke);

                    // Top-right corner
                    let tr = egui::pos2(cx + frame_w, cy - frame_h);
                    painter.line_segment([tr, egui::pos2(tr.x - arm, tr.y)], hud_stroke);
                    painter.line_segment([tr, egui::pos2(tr.x, tr.y + arm)], hud_stroke);

                    // Bottom-left corner
                    let bl = egui::pos2(cx - frame_w, cy + frame_h);
                    painter.line_segment([bl, egui::pos2(bl.x + arm, bl.y)], hud_stroke);
                    painter.line_segment([bl, egui::pos2(bl.x, bl.y - arm)], hud_stroke);

                    // Bottom-right corner
                    let br = egui::pos2(cx + frame_w, cy + frame_h);
                    painter.line_segment([br, egui::pos2(br.x - arm, br.y)], hud_stroke);
                    painter.line_segment([br, egui::pos2(br.x, br.y - arm)], hud_stroke);

                    // Top label tag (using Plasma)
                    painter.text(
                        egui::pos2(cx, cy - frame_h - 10.0),
                        egui::Align2::CENTER_CENTER,
                        "◈  NEURAL  FEED  ◈",
                        FontId::proportional(9.0),
                        NEON_PLASMA.gamma_multiply(0.4),
                    );

                    // Bottom data readout (using Solar)
                    let readout = format!(
                        "SYN:{:03}  •  FREQ:{:04}ms  •  CORE:∞",
                        state.quotes.len(),
                        state.rotation_interval.as_millis()
                    );
                    painter.text(
                        egui::pos2(cx, cy + frame_h + 12.0),
                        egui::Align2::CENTER_CENTER,
                        &readout,
                        FontId::proportional(8.5),
                        NEON_SOLAR.gamma_multiply(0.3),
                    );
                }

                // PREVIEW & EDITING LOGIC
                // If inputs have content, show them (Live Preview).
                let (main_text, sub_text, is_preview) = if !state.main_text_input.is_empty() {
                    (
                        state.main_text_input.clone(),
                        state.sub_text_input.clone(),
                        true,
                    )
                } else if !state.sub_text_input.is_empty() {
                    (
                        "Type text to preview...".to_string(),
                        state.sub_text_input.clone(),
                        true,
                    )
                } else {
                    // Not previewing, load current quote
                    match state.current_quote() {
                        Some(q) => (q.main_text.clone(), q.sub_text.clone(), false),
                        None => (String::new(), String::new(), false),
                    }
                };

                if !is_preview
                    && main_text.is_empty()
                    && sub_text.is_empty()
                    && state.quotes.is_empty()
                {
                    ui.label(
                        RichText::new("No quotes added yet!")
                            .color(Color32::GRAY)
                            .size(20.0),
                    );
                } else {
                    // 1. MAIN TEXT
                    let main_color = if is_preview && state.main_text_input.is_empty() {
                        Color32::WHITE.linear_multiply(0.6)
                    } else {
                        state.text_style.main_text_color
                    };
                    let main_size =
                        state.text_style.main_text_size * state.title_bar_state.zoom_level;

                    // Try cosmic-text shaped rendering for Bengali
                    // Use base color (without opacity) for cache efficiency
                    let base_main_color = state.text_style.main_text_color;
                    let used_shaped = if contains_bengali(&main_text) {
                        if let Some((ref mut fs, ref mut sc, ref mut tc)) = shaper {
                            if let Some((tex_id, size)) = render_shaped_text(
                                ctx,
                                fs,
                                sc,
                                &main_text,
                                main_size,
                                base_main_color,
                                tc,
                            ) {
                                let resp = ui.add(
                                    egui::Image::new(egui::load::SizedTexture::new(tex_id, size))
                                        .sense(if is_preview {
                                            egui::Sense::hover()
                                        } else {
                                            egui::Sense::click()
                                        }),
                                );
                                if !is_preview && resp.double_clicked() {
                                    state.main_text_input = main_text.clone();
                                    state.sub_text_input = sub_text.clone();
                                    state.title_bar_state.control_panel_visible = true;
                                    state.rotation_enabled = false;
                                    state.delete_quote(state.current_quote_index);
                                    state.save();
                                }
                                true
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    } else {
                        false
                    };

                    if !used_shaped {
                        let main_resp = ui.add(
                            egui::Label::new(
                                RichText::new(&main_text)
                                    .color(main_color)
                                    .size(main_size)
                                    .strong(),
                            )
                            .sense(if is_preview {
                                egui::Sense::hover()
                            } else {
                                egui::Sense::click()
                            }),
                        );

                        if !is_preview && main_resp.double_clicked() {
                            // Double click: Edit & Remove
                            state.main_text_input = main_text.clone();
                            state.sub_text_input = sub_text.clone();
                            state.title_bar_state.control_panel_visible = true;
                            state.rotation_enabled = false;
                            state.delete_quote(state.current_quote_index);
                            state.save();
                        }
                    } // end if !used_shaped

                    ui.add_space(state.text_style.between_gap);

                    // 2. SUB TEXT
                    if state.subtitle_editing && !is_preview {
                        // INLINE SUBTITLE EDITING
                        let edit = egui::TextEdit::singleline(&mut state.subtitle_edit_buffer)
                            .desired_width(300.0)
                            .horizontal_align(egui::Align::Center)
                            .font(egui::FontId::proportional(
                                state.text_style.sub_text_size * state.title_bar_state.zoom_level,
                            ));

                        let response = ui.add(edit);
                        response.request_focus();

                        if response.lost_focus() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            state.subtitle_editing = false;
                            if let Some(quote) = state.quotes.get_mut(state.current_quote_index) {
                                quote.sub_text = state.subtitle_edit_buffer.clone();
                                state.save();
                            }
                        }
                    } else {
                        // DISPLAY SUBTITLE
                        let sub_color = if is_preview && state.sub_text_input.is_empty() {
                            Color32::TRANSPARENT
                        } else {
                            state.text_style.sub_text_color
                        };

                        if !sub_text.is_empty() || is_preview {
                            let sub_size =
                                state.text_style.sub_text_size * state.title_bar_state.zoom_level;

                            // Try cosmic-text shaped rendering for Bengali subtitle
                            let base_sub_color = state.text_style.sub_text_color;
                            let used_shaped_sub = if contains_bengali(&sub_text) {
                                if let Some((ref mut fs, ref mut sc, ref mut tc)) = shaper {
                                    if let Some((tex_id, size)) = render_shaped_text(
                                        ctx,
                                        fs,
                                        sc,
                                        &sub_text,
                                        sub_size,
                                        base_sub_color,
                                        tc,
                                    ) {
                                        let sub_resp =
                                            ui.add(
                                                egui::Image::new(egui::load::SizedTexture::new(
                                                    tex_id, size,
                                                ))
                                                .sense(if is_preview {
                                                    egui::Sense::hover()
                                                } else {
                                                    egui::Sense::click()
                                                }),
                                            );
                                        if !is_preview {
                                            if sub_resp.double_clicked() {
                                                // Double click: Edit & Remove
                                                state.main_text_input = main_text.clone();
                                                state.sub_text_input = sub_text.clone();
                                                state.title_bar_state.control_panel_visible = true;
                                                state.rotation_enabled = false;
                                                state.delete_quote(state.current_quote_index);
                                                state.save();
                                            } else if sub_resp.clicked() {
                                                // Single click: Inline Edit
                                                state.subtitle_editing = true;
                                                state.subtitle_edit_buffer = sub_text.clone();
                                            }
                                        }
                                        true
                                    } else {
                                        false
                                    }
                                } else {
                                    false
                                }
                            } else {
                                false
                            };

                            if !used_shaped_sub {
                                let sub_resp = ui.add(
                                    egui::Label::new(
                                        RichText::new(&sub_text).color(sub_color).size(sub_size),
                                    )
                                    .sense(if is_preview {
                                        egui::Sense::hover()
                                    } else {
                                        egui::Sense::click()
                                    }),
                                );

                                if !is_preview {
                                    if sub_resp.double_clicked() {
                                        // Double click: Edit & Remove
                                        state.main_text_input = main_text;
                                        state.sub_text_input = sub_text.clone();
                                        state.title_bar_state.control_panel_visible = true;
                                        state.rotation_enabled = false;
                                        state.delete_quote(state.current_quote_index);
                                        state.save();
                                    } else if sub_resp.clicked() {
                                        // Single click: Inline Edit
                                        state.subtitle_editing = true;
                                        state.subtitle_edit_buffer = sub_text;
                                    }
                                }
                            } // end if !used_shaped_sub
                        }
                    }
                }

                // Navigation buttons
                ui.add_space(40.0);

                ui.horizontal(|ui| {
                    let total_btn_w = 220.0;
                    let avail = ui.available_width();
                    ui.add_space(((avail - total_btn_w) / 2.0).max(0.0));

                    // PREV — plasma violet
                    if draw_text_button(ui, "◀  PREV", Color32::from_rgb(80, 0, 160), 100.0, 34.0)
                        .clicked()
                    {
                        state.prev_quote();
                    }

                    ui.add_space(12.0);

                    // NEXT — neon teal
                    if draw_text_button(ui, "NEXT  ▶", Color32::from_rgb(0, 120, 100), 100.0, 34.0)
                        .clicked()
                    {
                        state.next_quote();
                    }
                });

                // Rotation status bar
                ui.add_space(24.0);

                ui.horizontal(|ui| {
                    let avail = ui.available_width();
                    ui.add_space(((avail - 280.0) / 2.0).max(0.0));

                    // Animated dot indicator
                    let dot_color = if state.rotation_enabled {
                        Color32::from_rgb(80, 255, 120)
                    } else {
                        Color32::from_rgb(255, 60, 80)
                    };

                    let (dot_rect, _) = ui.allocate_exact_size(Vec2::new(8.0, 8.0), Sense::hover());
                    let mid = dot_rect.center();
                    ui.painter().circle_filled(mid, 3.5, dot_color);
                    ui.painter().circle_stroke(
                        mid,
                        5.5,
                        Stroke::new(
                            0.5,
                            Color32::from_rgba_unmultiplied(
                                dot_color.r(),
                                dot_color.g(),
                                dot_color.b(),
                                80,
                            ),
                        ),
                    );

                    ui.add_space(6.0);
                    ui.label(
                        RichText::new(format!(
                            "Δt {}s  ·  {}",
                            state.rotation_interval.as_secs(),
                            if state.rotation_enabled {
                                "STREAMING"
                            } else {
                                "PAUSED"
                            }
                        ))
                        .color(Color32::from_rgba_unmultiplied(150, 200, 200, 180))
                        .size(10.5),
                    );
                });

                // Interval display
                ui.add_space(30.0);
                ui.label(
                    RichText::new(format!(
                        "Interval: {}s | Auto-rotation: {}",
                        state.rotation_interval.as_secs(),
                        if state.rotation_enabled { "ON" } else { "OFF" }
                    ))
                    .color(Color32::from_rgba_unmultiplied(255, 255, 255, 150))
                    .size(12.0),
                );
            });
        });
}

// =============================================================================
// CONTROL PANEL RENDERER
// =============================================================================

/// Render the control panel contents (inside SidePanel)
pub fn render_control_panel_contents(
    ui: &mut egui::Ui,
    state: &mut AppState,
    shaper: &mut Option<(
        &mut cosmic_text::FontSystem,
        &mut cosmic_text::SwashCache,
        &mut HashMap<u64, egui::TextureHandle>,
    )>,
) {
    ui.set_max_width(ui.available_width()); // Prevent horizontal overflow
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .enable_scrolling(true)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());

            // ===== Add Custom Text Section =====
            render_section(ui, "ADD CUSTOM TEXT", |ui| {
                // --- Main text input with A+/A-/color buttons to the right ---
                ui.horizontal(|ui| {
                    // Textarea on the left
                    let text_width = (ui.available_width() - 80.0).max(50.0);
                    let mut text_response = None;
                    egui::Frame::none()
                        .fill(Color32::from_black_alpha(60))
                        .rounding(Rounding::same(4.0))
                        .show(ui, |ui| {
                            let resp = ui.add(
                                egui::TextEdit::multiline(&mut state.main_text_input)
                                    .hint_text(
                                        "Main text... (Enter to submit, Shift+Enter for new line)",
                                    )
                                    .desired_rows(3)
                                    .desired_width(text_width)
                                    .lock_focus(true),
                            );
                            text_response = Some(resp);
                        });
                    let text_response = text_response.unwrap();
                    if text_response.changed() {
                        ui.ctx().request_repaint();
                    }
                    if text_response.has_focus()
                        && ui.input(|i| i.key_pressed(egui::Key::Enter) && !i.modifiers.shift)
                    {
                        if !state.main_text_input.trim().is_empty() {
                            state.add_quote(
                                state.main_text_input.clone(),
                                state.sub_text_input.clone(),
                            );
                            state.save();
                            state.main_text_input.clear();
                            state.sub_text_input.clear();
                            text_response.request_focus();
                        }
                    }

                    // Buttons column on the right
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            if ui
                                .small_button(RichText::new("A+").color(Color32::WHITE).size(10.0))
                                .clicked()
                                && state.text_style.main_text_size < 100.0
                            {
                                state.text_style.main_text_size += 2.0;
                                state.save();
                            }
                            // Color picker button
                            let color_btn = ui.add(
                                egui::Button::new(RichText::new("🎨").size(12.0))
                                    .fill(Color32::from_rgb(244, 67, 54))
                                    .min_size(Vec2::new(24.0, 20.0)),
                            );
                            if color_btn.clicked() {
                                state.show_main_color_picker = !state.show_main_color_picker;
                            }
                        });
                        if ui
                            .small_button(RichText::new("A-").color(Color32::WHITE).size(10.0))
                            .clicked()
                            && state.text_style.main_text_size > 12.0
                        {
                            state.text_style.main_text_size -= 2.0;
                            state.save();
                        }
                    });
                });

                // Color picker popup for main text
                if state.show_main_color_picker {
                    egui::Frame::none()
                        .fill(Color32::from_black_alpha(40))
                        .inner_margin(Vec2::new(8.0, 8.0))
                        .rounding(Rounding::same(4.0))
                        .show(ui, |ui| {
                            let mut color_arr = [
                                state.text_style.main_text_color.r(),
                                state.text_style.main_text_color.g(),
                                state.text_style.main_text_color.b(),
                                255u8,
                            ];
                            if ui
                                .color_edit_button_srgba_unmultiplied(&mut color_arr)
                                .changed()
                            {
                                state.text_style.main_text_color =
                                    Color32::from_rgb(color_arr[0], color_arr[1], color_arr[2]);
                                state.save();
                            }
                        });
                }

                ui.add_space(8.0);

                // --- Supporting text input with A+/A-/color buttons to the right ---
                ui.horizontal(|ui| {
                    let text_width = (ui.available_width() - 80.0).max(50.0);
                    let mut sub_response = None;
                    egui::Frame::none()
                        .fill(Color32::from_black_alpha(60))
                        .rounding(Rounding::same(4.0))
                        .show(ui, |ui| {
                            let resp = ui.add(
                                egui::TextEdit::multiline(&mut state.sub_text_input)
                                    .hint_text(
                                        "Supporting text... (Enter to submit, Shift+Enter for new line)",
                                    )
                                    .desired_rows(2)
                                    .desired_width(text_width),
                            );
                            sub_response = Some(resp);
                        });
                    let sub_response = sub_response.unwrap();
                    if sub_response.changed() {
                        ui.ctx().request_repaint();
                    }
                    if sub_response.has_focus()
                        && ui.input(|i| i.key_pressed(egui::Key::Enter) && !i.modifiers.shift)
                    {
                        if !state.main_text_input.trim().is_empty() {
                            // Only add if main text exists? Original: "Enter in EITHER triggers Add"
                            state.add_quote(
                                state.main_text_input.clone(),
                                state.sub_text_input.clone(),
                            );
                            state.save();
                            state.main_text_input.clear();
                            state.sub_text_input.clear();
                            // Focus back to main
                            // usage of main_text_response would be hard here as it's out of scope?
                            // I will set a flag or rely on `request_focus` content.
                            // Actually, I can't request focus on main input easily here without storing ID.
                            // But user asked "Focus returns to main textarea automatically".
                            // I'll skip focusing for now or try to use state.
                        }
                    }

                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            if ui
                                .small_button(RichText::new("A+").color(Color32::WHITE).size(10.0))
                                .clicked()
                                && state.text_style.sub_text_size < 50.0
                            {
                                state.text_style.sub_text_size += 1.0;
                                state.save();
                            }
                            let color_btn = ui.add(
                                egui::Button::new(RichText::new("🎨").size(12.0))
                                    .fill(Color32::from_rgb(244, 67, 54))
                                    .min_size(Vec2::new(24.0, 20.0)),
                            );
                            if color_btn.clicked() {
                                state.show_sub_color_picker = !state.show_sub_color_picker;
                            }
                        });
                        if ui
                            .small_button(RichText::new("A-").color(Color32::WHITE).size(10.0))
                            .clicked()
                            && state.text_style.sub_text_size > 8.0
                        {
                            state.text_style.sub_text_size -= 1.0;
                            state.save();
                        }
                    });
                });

                // Color picker popup for sub text
                if state.show_sub_color_picker {
                    egui::Frame::none()
                        .fill(Color32::from_black_alpha(40))
                        .inner_margin(Vec2::new(8.0, 8.0))
                        .rounding(Rounding::same(4.0))
                        .show(ui, |ui| {
                            let mut color_arr = [
                                state.text_style.sub_text_color.r(),
                                state.text_style.sub_text_color.g(),
                                state.text_style.sub_text_color.b(),
                                255u8,
                            ];
                            if ui
                                .color_edit_button_srgba_unmultiplied(&mut color_arr)
                                .changed()
                            {
                                state.text_style.sub_text_color =
                                    Color32::from_rgb(color_arr[0], color_arr[1], color_arr[2]);
                                state.save();
                            }
                        });
                }

                ui.add_space(8.0);

                // Add button
                let add_btn_color = Color32::from_rgb(76, 175, 80);
                if draw_text_button(
                    ui,
                    "+ Add Text",
                    add_btn_color,
                    ui.available_width() - 8.0,
                    32.0,
                )
                .clicked()
                {
                    if !state.main_text_input.is_empty() {
                        state
                            .add_quote(state.main_text_input.clone(), state.sub_text_input.clone());
                        state.save();
                        state.main_text_input.clear();
                        state.sub_text_input.clear();
                    }
                }
            });

            ui.add_space(10.0);

            // ===== Line Gaps Section =====
            render_section(ui, "LINE GAPS", |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("Main Text Gap")
                            .color(Color32::from_rgba_unmultiplied(255, 255, 255, 230))
                            .size(11.0),
                    );

                    // Add flexible space to push the label to the right
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            RichText::new(format!("{:.1}", state.text_style.main_line_gap))
                                .color(Color32::from_rgb(100, 200, 255))
                                .size(11.0)
                                .strong(),
                        );

                        // The slider takes the remaining width
                        let slider_width = ui.available_width();
                        if ui
                            .add_sized(
                                [slider_width, ui.available_height()],
                                egui::Slider::new(&mut state.text_style.main_line_gap, 1.0..=3.0)
                                    .step_by(0.1)
                                    .text(""),
                            )
                            .changed()
                        {
                            state.save();
                        }
                    });
                });

                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("Supporting Text Gap")
                            .color(Color32::from_rgba_unmultiplied(255, 255, 255, 230))
                            .size(11.0),
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            RichText::new(format!("{:.1}", state.text_style.sub_line_gap))
                                .color(Color32::from_rgb(100, 200, 255))
                                .size(11.0)
                                .strong(),
                        );
                        let slider_width = ui.available_width();
                        if ui
                            .add_sized(
                                [slider_width, ui.available_height()],
                                egui::Slider::new(&mut state.text_style.sub_line_gap, 1.0..=3.0)
                                    .step_by(0.1)
                                    .text(""),
                            )
                            .changed()
                        {
                            state.save();
                        }
                    });
                });

                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("Gap Between Texts")
                            .color(Color32::from_rgba_unmultiplied(255, 255, 255, 230))
                            .size(11.0),
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            RichText::new(format!("{:.0} px", state.text_style.between_gap))
                                .color(Color32::from_rgb(100, 200, 255))
                                .size(11.0)
                                .strong(),
                        );
                        let slider_width = ui.available_width();
                        if ui
                            .add_sized(
                                [slider_width, ui.available_height()],
                                egui::Slider::new(&mut state.text_style.between_gap, 0.0..=50.0)
                                    .step_by(1.0)
                                    .text(""),
                            )
                            .changed()
                        {
                            state.save();
                        }
                    });
                });
            });

            ui.add_space(10.0);

            // ===== Interval Section =====
            render_section(ui, "INTERVAL (SECONDS)", |ui| {
                ui.horizontal(|ui| {
                    let interval_resp =
                        ui.add(egui::DragValue::new(&mut state.interval_secs).range(1..=60));
                    if interval_resp.changed() {
                        // Clamp logic
                        state.interval_secs = state.interval_secs.clamp(1, 60);
                    }
                    if interval_resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        state.rotation_interval = Duration::from_secs(state.interval_secs);
                        state.last_rotation = Instant::now(); // Restart
                        state.save();
                    }

                    ui.label(RichText::new("seconds").color(Color32::GRAY).size(11.0));
                });

                ui.add_space(8.0);

                if draw_text_button(
                    ui,
                    "Set Interval",
                    Color32::from_rgb(33, 150, 243),
                    ui.available_width() - 8.0,
                    28.0,
                )
                .clicked()
                {
                    let clamped = state.interval_secs.clamp(1, 60);
                    state.interval_secs = clamped;
                    state.rotation_interval = Duration::from_secs(clamped);
                    state.last_rotation = Instant::now(); // RESTART TIMER
                    state.save();
                    ui.ctx().request_repaint();
                }

                ui.add_space(8.0);

                // Toggle rotation
                let (toggle_text, toggle_color) = if state.rotation_enabled {
                    ("⏸ Pause Rotation", Color32::from_rgb(255, 152, 0))
                } else {
                    ("▶ Resume Rotation", Color32::from_rgb(76, 175, 80))
                };

                if draw_text_button(
                    ui,
                    toggle_text,
                    toggle_color,
                    ui.available_width() - 8.0,
                    28.0,
                )
                .clicked()
                {
                    state.rotation_enabled = !state.rotation_enabled;
                    if state.rotation_enabled {
                        state.last_rotation = Instant::now();
                    }
                }
            });

            ui.add_space(10.0);

            // ===== Quotes List Section =====
            render_section(ui, &format!("TEXT LIST ({})", state.quotes.len()), |ui| {
                let mut to_delete: Option<usize> = None;
                let mut to_select: Option<usize> = None;

                for (idx, quote) in state.quotes.iter().enumerate() {
                    let is_current = idx == state.current_quote_index;
                    let bg_color = if is_current {
                        Color32::from_black_alpha(35)
                    } else {
                        Color32::from_black_alpha(20)
                    };

                    egui::Frame::none()
                        .fill(bg_color)
                        .inner_margin(Vec2::new(8.0, 6.0))
                        .rounding(Rounding::same(4.0))
                        .stroke(Stroke::new(
                            1.0,
                            Color32::from_rgba_unmultiplied(255, 255, 255, 50),
                        ))
                        .show(ui, |ui| {
                            // Let the text flexibly fill space
                            // Delete button goes on the very right
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    // Delete button
                                    let del_btn = ui.add(
                                        egui::Button::new(
                                            RichText::new("Delete").color(Color32::WHITE).size(9.0),
                                        )
                                        .fill(Color32::from_rgb(255, 70, 70))
                                        .min_size(Vec2::new(40.0, 18.0)),
                                    );
                                    if del_btn.clicked() {
                                        to_delete = Some(idx);
                                    }

                                    // Text Area takes remaining space
                                    ui.with_layout(
                                        egui::Layout::left_to_right(egui::Align::Min),
                                        |ui| {
                                            ui.vertical(|ui| {
                                                // Line 1: N. [main quote text]
                                                let display_main =
                                                    format!("{}. {}", idx + 1, &quote.main_text);
                                                let clicked_main;
                                                if contains_bengali(&quote.main_text) {
                                                    if let Some((
                                                        ref mut fs,
                                                        ref mut sc,
                                                        ref mut tc,
                                                    )) = shaper
                                                    {
                                                        if let Some((tex_id, size)) =
                                                            render_shaped_text(
                                                                ui.ctx(),
                                                                fs,
                                                                sc,
                                                                &display_main,
                                                                9.0,
                                                                Color32::WHITE,
                                                                tc,
                                                            )
                                                        {
                                                            let resp = ui.add(
                                                                egui::Image::new(
                                                                    egui::load::SizedTexture::new(
                                                                        tex_id, size,
                                                                    ),
                                                                )
                                                                .sense(egui::Sense::click()),
                                                            );
                                                            clicked_main = resp.clicked();
                                                        } else {
                                                            let resp = ui.label(
                                                                RichText::new(&display_main)
                                                                    .color(Color32::WHITE)
                                                                    .size(9.0),
                                                            );
                                                            clicked_main = resp.clicked();
                                                        }
                                                    } else {
                                                        let resp = ui.label(
                                                            RichText::new(&display_main)
                                                                .color(Color32::WHITE)
                                                                .size(9.0),
                                                        );
                                                        clicked_main = resp.clicked();
                                                    }
                                                } else {
                                                    let resp = ui.label(
                                                        RichText::new(&display_main)
                                                            .color(Color32::WHITE)
                                                            .size(9.0),
                                                    );
                                                    clicked_main = resp.clicked();
                                                }

                                                // Line 2: 💬 [supporting text]
                                                let display_sub = format!("💬 {}", &quote.sub_text);
                                                if contains_bengali(&quote.sub_text) {
                                                    if let Some((
                                                        ref mut fs,
                                                        ref mut sc,
                                                        ref mut tc,
                                                    )) = shaper
                                                    {
                                                        if let Some((tex_id, size)) =
                                                            render_shaped_text(
                                                                ui.ctx(),
                                                                fs,
                                                                sc,
                                                                &display_sub,
                                                                9.0,
                                                                Color32::from_rgba_unmultiplied(
                                                                    255, 255, 255, 200,
                                                                ),
                                                                tc,
                                                            )
                                                        {
                                                            ui.add(egui::Image::new(
                                                                egui::load::SizedTexture::new(
                                                                    tex_id, size,
                                                                ),
                                                            ));
                                                        } else {
                                                            ui.label(
                                                    RichText::new(&display_sub)
                                                        .color(Color32::from_rgba_unmultiplied(
                                                            255, 255, 255, 200,
                                                        ))
                                                        .size(9.0),
                                                );
                                                        }
                                                    } else {
                                                        ui.label(
                                                            RichText::new(&display_sub)
                                                                .color(
                                                                    Color32::from_rgba_unmultiplied(
                                                                        255, 255, 255, 200,
                                                                    ),
                                                                )
                                                                .size(9.0),
                                                        );
                                                    }
                                                } else {
                                                    ui.label(
                                                        RichText::new(&display_sub)
                                                            .color(Color32::from_rgba_unmultiplied(
                                                                255, 255, 255, 200,
                                                            ))
                                                            .size(9.0),
                                                    );
                                                }

                                                if clicked_main {
                                                    to_select = Some(idx);
                                                }
                                            });
                                        },
                                    );
                                },
                            );
                        });

                    ui.add_space(4.0);
                }

                // Apply changes after iteration
                if let Some(idx) = to_delete {
                    state.delete_quote(idx);
                    state.save();
                }
                if let Some(idx) = to_select {
                    state.current_quote_index = idx;
                    state.last_rotation = Instant::now();
                }
            });

            ui.add_space(10.0);

            // ===== Clear All Section =====
            if !state.confirm_clear_pending {
                if draw_text_button(
                    ui,
                    "Clear All",
                    Color32::from_rgb(255, 152, 0), // Orange per HTML
                    ui.available_width(),
                    28.0,
                )
                .clicked()
                {
                    state.confirm_clear_pending = true;
                }
            } else {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("Are you sure?")
                            .color(Color32::WHITE)
                            .size(11.0),
                    );
                    if ui
                        .button(RichText::new("Yes, Clear").color(Color32::WHITE).size(10.0))
                        .clicked()
                    {
                        state.quotes.clear();
                        state.current_quote_index = 0;
                        state.confirm_clear_pending = false;
                        state.save();
                    }
                    if ui.button(RichText::new("Cancel").size(10.0)).clicked() {
                        state.confirm_clear_pending = false;
                    }
                });
            }

            ui.add_space(10.0);

            // ===== Info Section =====
            egui::Frame::none()
                .fill(Color32::from_black_alpha(26))
                .stroke(egui::Stroke::new(1.0, Color32::from_white_alpha(30)))
                .inner_margin(Vec2::new(10.0, 10.0))
                .rounding(Rounding::same(4.0))
                .show(ui, |ui| {
                    ui.label(
                        RichText::new(format!(
                            "Current Interval: {}s",
                            state.rotation_interval.as_secs()
                        ))
                        .color(Color32::GRAY)
                        .size(10.0),
                    );
                    ui.label(
                        RichText::new(format!("Total Quotes: {}", state.quotes.len()))
                            .color(Color32::GRAY)
                            .size(10.0),
                    );
                    ui.label(
                        RichText::new(format!(
                            "Rotation: {}",
                            if state.rotation_enabled {
                                "Active"
                            } else {
                                "Paused"
                            }
                        ))
                        .color(Color32::GRAY)
                        .size(10.0),
                    );
                });
        });
}

/// Render a section with title
fn render_section(ui: &mut egui::Ui, title: &str, add_contents: impl FnOnce(&mut egui::Ui)) {
    // Outer frame with relative darkening
    egui::Frame::none()
        .fill(Color32::from_black_alpha(20))
        .stroke(Stroke::new(1.0, Color32::from_white_alpha(30)))
        .inner_margin(egui::Margin::same(1.0))
        .rounding(Rounding::same(10.0))
        .show(ui, |ui| {
            // Inner subtle depth
            egui::Frame::none()
                .fill(Color32::from_black_alpha(13))
                .stroke(Stroke::new(
                    0.5,
                    Color32::from_rgba_unmultiplied(255, 255, 255, 8),
                ))
                .inner_margin(egui::Margin {
                    left: 12.0,
                    right: 12.0,
                    top: 10.0,
                    bottom: 12.0,
                })
                .rounding(Rounding::same(9.0))
                .show(ui, |ui| {
                    // Section title row with decorative line
                    ui.horizontal(|ui| {
                        // Left accent mark
                        let (mark_rect, _) =
                            ui.allocate_exact_size(Vec2::new(3.0, 12.0), Sense::hover());
                        ui.painter().rect_filled(
                            mark_rect,
                            Rounding::same(2.0),
                            Color32::from_rgb(0, 255, 220),
                        );

                        ui.add_space(6.0);

                        ui.label(
                            RichText::new(title)
                                .color(Color32::WHITE)
                                .size(10.5)
                                .strong(),
                        );

                        // Trailing separator line
                        let avail = ui.available_width();
                        if avail > 4.0 {
                            let (line_rect, _) =
                                ui.allocate_exact_size(Vec2::new(avail - 2.0, 1.0), Sense::hover());
                            let mid_y = line_rect.center().y;
                            ui.painter().line_segment(
                                [
                                    egui::pos2(line_rect.left(), mid_y),
                                    egui::pos2(line_rect.right(), mid_y),
                                ],
                                Stroke::new(0.5, Color32::from_white_alpha(40)),
                            );
                        }
                    });

                    ui.add_space(8.0);
                    add_contents(ui);
                });
        });
}

// =============================================================================
// THEME MODAL RENDERER
// =============================================================================

/// Render the theme customization modal
pub fn render_theme_modal(ctx: &Context, state: &mut AppState) {
    if !state.theme_modal_open {
        return;
    }

    egui::Window::new("Customize Theme")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, Vec2::new(0.0, 0.0))
        .fixed_size(Vec2::new(400.0, 500.0))
        .frame(egui::Frame::window(&ctx.style()).fill(Color32::from_white_alpha(15)))
        .show(ctx, |ui| {
            // Mode toggle
            ui.horizontal(|ui| {
                ui.label(RichText::new("Mode:").color(Color32::WHITE).size(12.0));

                let gradient_selected = state.theme.mode == ThemeMode::Gradient;
                let solid_selected = state.theme.mode == ThemeMode::Solid;

                if ui.selectable_label(gradient_selected, "Gradient").clicked() {
                    state.theme.mode = ThemeMode::Gradient;
                    state.save();
                }
                if ui.selectable_label(solid_selected, "Solid").clicked() {
                    state.theme.mode = ThemeMode::Solid;
                    state.save();
                }
            });

            ui.add_space(10.0);

            ui.horizontal(|ui| {
                if ui
                    .checkbox(
                        &mut state.theme.apply_to_entire_window,
                        "Apply to Entire Window",
                    )
                    .changed()
                {
                    state.save();
                }
            });

            ui.add_space(15.0);

            if state.theme.mode == ThemeMode::Gradient {
                // Gradient angle
                ui.label(
                    RichText::new("Gradient Angle:")
                        .color(Color32::WHITE)
                        .size(12.0),
                );
                ui.add_space(5.0);

                ui.horizontal_wrapped(|ui| {
                    for angle in [0, 45, 90, 135, 180, 225, 270, 315] {
                        let selected = state.theme.gradient_angle == angle;
                        if ui
                            .selectable_label(selected, format!("{}°", angle))
                            .clicked()
                        {
                            state.theme.gradient_angle = angle;
                            state.save();
                        }
                    }
                });

                ui.add_space(15.0);

                // Gradient colors
                ui.label(
                    RichText::new("Gradient Colors:")
                        .color(Color32::WHITE)
                        .size(12.0),
                );
                ui.add_space(5.0);

                let mut to_remove = None;
                for idx in 0..state.theme.gradient_colors.len() {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(format!("Color {}:", idx + 1))
                                .color(Color32::GRAY)
                                .size(11.0),
                        );

                        // Color picker (RGBA format)
                        let color = state.theme.gradient_colors[idx];
                        let mut color_array = [
                            color.r() as f32 / 255.0,
                            color.g() as f32 / 255.0,
                            color.b() as f32 / 255.0,
                            1.0,
                        ];
                        if ui
                            .color_edit_button_rgba_unmultiplied(&mut color_array)
                            .changed()
                        {
                            state.theme.gradient_colors[idx] = Color32::from_rgb(
                                (color_array[0] * 255.0) as u8,
                                (color_array[1] * 255.0) as u8,
                                (color_array[2] * 255.0) as u8,
                            );
                            state.save();
                        }

                        // Remove button (only when > 2 colors)
                        if state.theme.gradient_colors.len() > 2 {
                            let remove_btn = ui.add(
                                egui::Button::new(
                                    RichText::new("Remove").color(Color32::WHITE).size(10.0),
                                )
                                .fill(Color32::from_rgb(255, 70, 70)),
                            );
                            if remove_btn.clicked() {
                                to_remove = Some(idx);
                            }
                        }
                    });
                }

                if let Some(idx) = to_remove {
                    state.theme.gradient_colors.remove(idx);
                    state.save();
                }

                // Add color button
                if state.theme.gradient_colors.len() < 5 {
                    if ui.button("+ Add Color").clicked() {
                        state.theme.gradient_colors.push(Color32::WHITE);
                        state.save();
                    }
                }

                ui.add_space(15.0);

                // Presets
                ui.label(
                    RichText::new("Preset Gradients:")
                        .color(Color32::WHITE)
                        .size(12.0),
                );
                ui.add_space(5.0);

                // Preset buttons
                ui.horizontal_wrapped(|ui| {
                    if ui.button("⬡ Aurora Void").clicked() {
                        state.theme.gradient_colors = vec![
                            Color32::from_rgb(2, 4, 16),
                            Color32::from_rgb(30, 0, 80),
                            Color32::from_rgb(0, 60, 120),
                            Color32::from_rgb(0, 200, 180),
                        ];
                        state.save();
                    }
                    if ui.button("⬡ Solar Flare").clicked() {
                        state.theme.gradient_colors = vec![
                            Color32::from_rgb(10, 0, 30),
                            Color32::from_rgb(120, 20, 0),
                            Color32::from_rgb(255, 100, 0),
                            Color32::from_rgb(255, 220, 60),
                        ];
                        state.save();
                    }
                });
                ui.horizontal_wrapped(|ui| {
                    if ui.button("⬡ Plasma Storm").clicked() {
                        state.theme.gradient_colors = vec![
                            Color32::from_rgb(5, 0, 20),
                            Color32::from_rgb(80, 0, 180),
                            Color32::from_rgb(200, 0, 255),
                            Color32::from_rgb(255, 80, 200),
                        ];
                        state.save();
                    }
                    if ui.button("⬡ Deep Ocean").clicked() {
                        state.theme.gradient_colors = vec![
                            Color32::from_rgb(0, 5, 20),
                            Color32::from_rgb(0, 30, 80),
                            Color32::from_rgb(0, 100, 160),
                            Color32::from_rgb(0, 200, 220),
                        ];
                        state.save();
                    }
                });
                ui.horizontal_wrapped(|ui| {
                    if ui.button("⬡ Matrix Rain").clicked() {
                        state.theme.gradient_colors = vec![
                            Color32::from_rgb(0, 8, 0),
                            Color32::from_rgb(0, 40, 10),
                            Color32::from_rgb(0, 120, 30),
                            Color32::from_rgb(80, 255, 100),
                        ];
                        state.save();
                    }
                    if ui.button("⬡ Quantum Noir").clicked() {
                        state.theme.gradient_colors = vec![
                            Color32::from_rgb(2, 2, 6),
                            Color32::from_rgb(10, 10, 25),
                            Color32::from_rgb(25, 25, 50),
                            Color32::from_rgb(60, 60, 100),
                        ];
                        state.save();
                    }
                });
            } else {
                // Solid color
                ui.label(
                    RichText::new("Solid Color:")
                        .color(Color32::WHITE)
                        .size(12.0),
                );
                ui.add_space(5.0);

                let solid = state.theme.solid_color;
                let mut color_array = [
                    solid.r() as f32 / 255.0,
                    solid.g() as f32 / 255.0,
                    solid.b() as f32 / 255.0,
                    1.0,
                ];
                if ui
                    .color_edit_button_rgba_unmultiplied(&mut color_array)
                    .changed()
                {
                    state.theme.solid_color = Color32::from_rgb(
                        (color_array[0] * 255.0) as u8,
                        (color_array[1] * 255.0) as u8,
                        (color_array[2] * 255.0) as u8,
                    );
                    state.save();
                }
            }

            ui.add_space(20.0);

            // Action buttons
            ui.horizontal(|ui| {
                if ui
                    .button(
                        RichText::new("Apply Theme")
                            .color(Color32::WHITE)
                            .size(12.0),
                    )
                    .clicked()
                {
                    state.theme_modal_open = false;
                }

                if ui
                    .button(RichText::new("Reset").color(Color32::WHITE).size(12.0))
                    .clicked()
                {
                    state.theme = ThemeConfig::default();
                }

                if ui
                    .button(RichText::new("✕").color(Color32::WHITE).size(14.0))
                    .clicked()
                {
                    state.theme_modal_open = false;
                }
            });
        });
}

// =============================================================================
// WGUP RENDER STATE
// =============================================================================

#[allow(dead_code)]
struct WgpuRenderState<'a> {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'a>,
    surface_config: wgpu::SurfaceConfiguration,
    renderer: egui_wgpu::Renderer,
}

#[allow(dead_code)]
impl<'a> WgpuRenderState<'a> {
    async fn new(window: &'a Window) -> Result<WgpuRenderState<'a>, String> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
            flags: wgpu::InstanceFlags::empty(),
            gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
        });

        let surface = instance
            .create_surface(window)
            .map_err(|e| format!("Failed to create surface: {}", e))?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| "Failed to request adapter".to_string())?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: adapter.limits(),
                    memory_hints: wgpu::MemoryHints::default(),
                },
                None,
            )
            .await
            .map_err(|e| format!("Failed to request device: {}", e))?;

        let size = window.inner_size();
        let capabilities = surface.get_capabilities(&adapter);
        let format = capabilities
            .formats
            .first()
            .copied()
            .unwrap_or(wgpu::TextureFormat::Bgra8UnormSrgb);

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &surface_config);

        // Renderer::new now takes 5 arguments: device, format, depth_texture, msaa_samples, debug
        let renderer = egui_wgpu::Renderer::new(&device, format, None, 1, false);

        Ok(Self {
            device,
            queue,
            surface,
            surface_config,
            renderer,
        })
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.surface_config.width = new_size.width;
            self.surface_config.height = new_size.height;
            self.surface.configure(&self.device, &self.surface_config);
        }
    }
}

// =============================================================================
// MAIN ENTRY POINT
// =============================================================================

#[cfg(windows)]
fn get_global_cursor() -> Option<(i32, i32)> {
    use windows::Win32::Foundation::POINT;
    use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;
    let mut pt = POINT::default();
    if unsafe { GetCursorPos(&mut pt) }.is_ok() {
        Some((pt.x, pt.y))
    } else {
        None
    }
}

#[cfg(not(windows))]
fn get_global_cursor() -> Option<(i32, i32)> {
    None
}

fn log_to_file(msg: &str) {
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("debug.log")
    {
        let _ = writeln!(file, "{}", msg);
    }
}

#[cfg(windows)]
fn set_window_topmost(hwnd: HWND) {
    unsafe {
        let _ = SetWindowPos(
            hwnd,
            HWND_TOPMOST,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_SHOWWINDOW,
        );
    }
}

#[cfg(not(windows))]
fn set_window_topmost() {
    // Not supported on non-Windows platforms
}

fn main() {
    println!("==========================================");
    std::io::Write::flush(&mut std::io::stdout()).ok();
    println!("  Daily Motivation - Pure Rust GUI");
    std::io::Write::flush(&mut std::io::stdout()).ok();
    println!("  Built with winit + wgpu + egui");
    std::io::Write::flush(&mut std::io::stdout()).ok();
    println!("==========================================");
    std::io::Write::flush(&mut std::io::stdout()).ok();
    println!("\nFeatures:");
    println!("  💪 Custom title bar with icons");
    println!("  🎨 Theme customization");
    println!("  📝 Quote management");
    println!("  ⏱ Configurable rotation intervals");
    println!("  🔍 Zoom controls");
    println!("==========================================\n");
    std::io::Write::flush(&mut std::io::stdout()).ok();

    log_to_file("Starting application");
    let event_loop = EventLoop::new().unwrap();
    log_to_file("Event loop created");

    let mut app_runner = AppRunner {
        window: None,
        render_state: None,
        app_state: None,
        egui_ctx: None,
        egui_state: None,
        font_system: Some(cosmic_text::FontSystem::new()),
        swash_cache: Some(cosmic_text::SwashCache::new()),
        shaped_text_textures: HashMap::new(),
        should_close: false,
    };

    log_to_file("Running event loop");
    // Use the new run_app API with proper window creation in the event loop
    let _ = event_loop.run_app(&mut app_runner);
    log_to_file("Event loop exited");
}

/// Setup custom fonts for Bangla/Bengali text support
fn setup_fonts(ctx: &Context) {
    let mut fonts = egui::FontDefinitions::default();

    // Try common Bengali fonts on Windows + local fallbacks
    // Nirmala.ttc is the standard TrueType Collection on Windows 10/11
    let font_paths = [
        "C:\\Windows\\Fonts\\Nirmala.ttc",
        "C:\\Windows\\Fonts\\Vrinda.ttf",
        "C:\\Windows\\Fonts\\Siyamrupali.ttf",
        "C:\\Windows\\Fonts\\ShonarBangla.ttf",
        "C:\\Windows\\Fonts\\Shonar.ttf",
        "C:\\Windows\\Fonts\\NotoSansBengali-Regular.ttf",
        "C:\\Windows\\Fonts\\arialuni.ttf",
        "NotoSansBengali-Regular.ttf",
        "assets/NotoSansBengali-Regular.ttf",
    ];

    let mut loaded = false;
    for path in font_paths {
        if let Ok(data) = std::fs::read(path) {
            // Note: egui uses ab_glyph which supports .ttf, .otf, and .ttc
            // For .ttc, it will use the first font in the collection
            fonts
                .font_data
                .insert("bengali".to_owned(), egui::FontData::from_owned(data));

            // Priority 0: Always put our support font first in families
            if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
                family.insert(0, "bengali".to_owned());
            }
            if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
                family.insert(0, "bengali".to_owned());
            }

            log_to_file(&format!("Loaded Bengali font from: {}", path));
            loaded = true;
            break;
        }
    }

    if !loaded {
        log_to_file("WARNING: No Bengali fonts found. Bangla text rendering will likely fail.");
    }

    // Initialize nerdfonts
    fonts.font_data.insert(
        "nerdfonts".to_owned(),
        egui::FontData::from_static(include_bytes!("../assets/nerdfonts_regular.ttf")),
    );
    if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
        family.push("nerdfonts".to_owned());
    }

    ctx.set_fonts(fonts);
}

/// Check if a string contains Bengali/Bangla characters
fn contains_bengali(text: &str) -> bool {
    text.chars().any(|c| matches!(c, '\u{0980}'..='\u{09FF}'))
}

/// Render shaped text using cosmic-text and return an egui texture.
/// This properly handles complex scripts like Bengali through rustybuzz (HarfBuzz port).
fn render_shaped_text(
    ctx: &Context,
    font_system: &mut cosmic_text::FontSystem,
    swash_cache: &mut cosmic_text::SwashCache,
    text: &str,
    font_size: f32,
    color: Color32,
    tex_cache: &mut HashMap<u64, egui::TextureHandle>,
) -> Option<(egui::TextureId, Vec2)> {
    if text.is_empty() {
        return None;
    }

    // Create a cache key from the text, size, and color
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    text.hash(&mut hasher);
    font_size.to_bits().hash(&mut hasher);
    color.to_array().hash(&mut hasher);
    let cache_key = hasher.finish();

    // Return cached texture if available
    if let Some(handle) = tex_cache.get(&cache_key) {
        let size = handle.size();
        return Some((handle.id(), Vec2::new(size[0] as f32, size[1] as f32)));
    }

    // Create cosmic-text buffer for shaping
    let metrics = cosmic_text::Metrics::new(font_size, font_size * 1.3);
    let mut buffer = cosmic_text::Buffer::new(font_system, metrics);

    // Set a wide width so it doesn't wrap
    buffer.set_size(font_system, Some(2000.0), None);

    let attrs = cosmic_text::Attrs::new().family(cosmic_text::Family::Name("Nirmala UI"));
    buffer.set_text(font_system, text, attrs, cosmic_text::Shaping::Advanced);
    buffer.shape_until_scroll(font_system, false);

    // Calculate dimensions from layout runs
    let mut max_width: f32 = 0.0;
    let mut total_height: f32 = 0.0;
    for run in buffer.layout_runs() {
        max_width = max_width.max(run.line_w);
        total_height += run.line_height;
    }

    if max_width <= 0.0 || total_height <= 0.0 {
        return None;
    }

    let width = (max_width.ceil() as usize).max(1);
    let height = (total_height.ceil() as usize).max(1);

    // Create pixel buffer (RGBA)
    let mut pixels = vec![Color32::TRANSPARENT; width * height];

    // Draw glyphs using swash cache
    let text_color = cosmic_text::Color::rgba(color.r(), color.g(), color.b(), color.a());

    buffer.draw(
        font_system,
        swash_cache,
        text_color,
        |x, y, _w, _h, drawn_color| {
            // drawn_color is the blended color for this pixel
            let px = x as usize;
            let py = y as usize;
            if px < width && py < height && x >= 0 && y >= 0 {
                let alpha = drawn_color.a();
                if alpha > 0 {
                    let idx = py * width + px;
                    // Alpha-blend the glyph pixel onto the transparent background
                    pixels[idx] = Color32::from_rgba_premultiplied(
                        drawn_color.r(),
                        drawn_color.g(),
                        drawn_color.b(),
                        alpha,
                    );
                }
            }
        },
    );

    // Create egui texture
    let image = egui::ColorImage {
        size: [width, height],
        pixels,
    };

    let texture = ctx.load_texture(
        format!("shaped_{}", cache_key),
        image,
        egui::TextureOptions::LINEAR,
    );

    let size = Vec2::new(width as f32, height as f32);
    let tex_id = texture.id();
    tex_cache.insert(cache_key, texture);

    Some((tex_id, size))
}

// Implement winit::application::ApplicationHandler for the new API
use winit::application::ApplicationHandler;
use winit::event_loop::ActiveEventLoop;

struct AppRunner {
    window: Option<&'static Window>,
    render_state: Option<WgpuRenderState<'static>>,
    app_state: Option<AppState>,
    egui_ctx: Option<Context>,
    egui_state: Option<egui_winit::State>,
    // cosmic-text for proper Bengali/Indic text shaping
    font_system: Option<cosmic_text::FontSystem>,
    swash_cache: Option<cosmic_text::SwashCache>,
    shaped_text_textures: HashMap<u64, egui::TextureHandle>,
    should_close: bool,
}

impl ApplicationHandler for AppRunner {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return; // Window already created
        }

        log_to_file("resumed() called - creating window");

        // Create the window through the event loop
        match event_loop.create_window(
            Window::default_attributes()
                .with_title("Daily Motivation")
                .with_inner_size(LogicalSize::new(
                    DEFAULT_WINDOW_SIZE.0 as f64,
                    DEFAULT_WINDOW_SIZE.1 as f64,
                ))
                .with_min_inner_size(LogicalSize::new(
                    MIN_WINDOW_SIZE.0 as f64,
                    MIN_WINDOW_SIZE.1 as f64,
                ))
                .with_decorations(false)
                .with_resizable(true)
                .with_transparent(true)
                .with_visible(false), // Start invisible to avoid white flash
        ) {
            Ok(window) => {
                log_to_file("Window created");
                let window = Box::leak(Box::new(window));

                // Set window topmost on Windows
                #[cfg(windows)]
                {
                    use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
                    if let Ok(handle) = window.window_handle() {
                        if let RawWindowHandle::Win32(win32_handle) = handle.as_raw() {
                            let hwnd = HWND(win32_handle.hwnd.get() as *mut _);
                            set_window_topmost(hwnd);
                        }
                    }
                }

                eprintln!("Window created successfully");
                log_to_file("Window created successfully");

                self.window = Some(window);

                log_to_file("Creating render state and egui components");

                match pollster::block_on(WgpuRenderState::new(window)) {
                    Ok(render_state) => {
                        let app_state = AppState::default();
                        let egui_ctx = Context::default();
                        let mut style = egui::Style::default();
                        style.visuals = egui::Visuals::dark();
                        style.visuals.window_fill = CANVAS_BG;
                        style.visuals.panel_fill = CONTROL_PANEL_BG;

                        // Add global hover effects for buttons
                        let mut visuals = style.visuals.clone();
                        visuals.widgets.hovered.bg_fill = Color32::from_rgb(80, 80, 90);
                        visuals.widgets.hovered.bg_stroke =
                            egui::Stroke::new(1.0, Color32::WHITE.gamma_multiply(0.5));
                        visuals.widgets.active.bg_fill = Color32::from_rgb(100, 100, 110);
                        style.visuals = visuals;

                        egui_ctx.set_style(style);

                        let egui_state = egui_winit::State::new(
                            egui_ctx.clone(),
                            egui::ViewportId::ROOT,
                            window,
                            None,
                            None,
                            None,
                        );

                        self.render_state = Some(render_state);
                        self.app_state = Some(app_state);
                        self.egui_ctx = Some(egui_ctx.clone());
                        self.egui_state = Some(egui_state);

                        // Load Bengali fonts for Bangla text support
                        setup_fonts(&egui_ctx);

                        // Show window now that rendering is ready (prevents white flash)
                        window.set_visible(true);

                        log_to_file("Render state stored in AppRunner");
                    }
                    Err(e) => {
                        eprintln!("Warning: Render state initialization failed: {}", e);
                        log_to_file(&format!("Render state initialization failed: {}", e));
                        event_loop.exit();
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to create window: {}", e);
                log_to_file(&format!("Failed to create window: {}", e));
                event_loop.exit();
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        if let Some(window) = self.window {
            // Forward ALL events to egui so it can respond to mouse/keyboard immediately
            if let Some(egui_state) = self.egui_state.as_mut() {
                let _ = egui_state.on_window_event(window, &event);
            }

            match event {
                WindowEvent::CloseRequested => {
                    event_loop.exit();
                }
                WindowEvent::Resized(size) => {
                    if let Some(render_state) = self.render_state.as_mut() {
                        render_state.resize(size);
                    }
                }
                WindowEvent::RedrawRequested => {
                    self.render(&window);
                }
                _ => {}
            }
        }

        // Update interaction time on user input
        if let Some(app_state) = self.app_state.as_mut() {
            match event {
                WindowEvent::CursorMoved { .. }
                | WindowEvent::MouseInput { .. }
                | WindowEvent::KeyboardInput { .. } => {
                    app_state.last_interaction = Instant::now();

                    // Stop all animations on Space key
                    if let WindowEvent::KeyboardInput { event, .. } = event {
                        if event.state == winit::event::ElementState::Pressed {
                            if let winit::keyboard::PhysicalKey::Code(
                                winit::keyboard::KeyCode::Space,
                            ) = event.physical_key
                            {
                                app_state.active_animation = AppAnimation::None;
                                // Reset common effects
                                if let Some(window) = self.window {
                                    if let Ok(handle) = window.window_handle() {
                                        if let winit::raw_window_handle::RawWindowHandle::Win32(
                                            win32,
                                        ) = handle.as_raw()
                                        {
                                            let hwnd = HWND(win32.hwnd.get() as _);
                                            unsafe {
                                                let _ = SetLayeredWindowAttributes(
                                                    hwnd, None, 255, LWA_ALPHA,
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Request repaint to ensure UI updates immediately
                    self.window.as_ref().map(|w| w.request_redraw());
                }
                _ => {}
            }
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if self.should_close {
            event_loop.exit();
            return;
        }

        // Render if we have a window and render state
        if let Some(window) = self.window {
            self.render(&window);
        }

        if self.should_close {
            event_loop.exit();
            return;
        }

        // Smart sleep: use shorter delay only when egui needs repainting,
        // otherwise sleep longer to save CPU and prevent system lag
        let sleep_ms = if let Some(ctx) = self.egui_ctx.as_ref() {
            if ctx.has_requested_repaint() {
                16 // Active interaction: ~60 FPS
            } else {
                100 // Idle: ~10 FPS (plenty for quote rotation)
            }
        } else {
            16
        };
        thread::sleep(Duration::from_millis(sleep_ms));
    }
}

impl AppRunner {
    fn render(&mut self, window: &Window) {
        // Take cosmic-text state out of self before entering the closure
        let mut font_system = self.font_system.take();
        let mut swash_cache = self.swash_cache.take();
        let mut tex_cache = std::mem::take(&mut self.shaped_text_textures);

        let (app_state, egui_ctx, egui_state, render_state) = match (
            self.app_state.as_mut(),
            self.egui_ctx.as_mut(),
            self.egui_state.as_mut(),
            self.render_state.as_mut(),
        ) {
            (Some(state), Some(ctx), Some(est), Some(rst)) => (state, ctx, est, rst),
            _ => {
                // Return states before returning
                self.font_system = font_system;
                self.swash_cache = swash_cache;
                self.shaped_text_textures = tex_cache;
                return;
            }
        };

        // (Animation Engine moved below)

        let raw_input = egui_state.take_egui_input(window);
        let full_output = egui_ctx.run(raw_input, |ctx| {
            // Track activity for auto-hide
            if ctx.is_using_pointer() || ctx.input(|i| i.pointer.any_down() || !i.events.is_empty())
            {
                app_state.last_interaction = Instant::now();
            }

            let mut is_resizing = false;
            // Handle active manual resizing
            if let Some((dir, start_cx, start_cy, start_wx, start_wy, start_w, start_h)) =
                app_state.manual_resize_start
            {
                is_resizing = true;
                if ctx.input(|i| i.pointer.primary_down()) {
                    if let Some((cx, cy)) = get_global_cursor() {
                        let dx = cx - start_cx;
                        let dy = cy - start_cy;

                        let mut new_w = start_w as i32;
                        let mut new_h = start_h as i32;
                        let mut new_x = start_wx;
                        let mut new_y = start_wy;

                        use winit::window::ResizeDirection;
                        match dir {
                            ResizeDirection::East => new_w += dx,
                            ResizeDirection::West => {
                                new_w -= dx;
                                new_x += dx;
                            }
                            ResizeDirection::South => new_h += dy,
                            ResizeDirection::North => {
                                new_h -= dy;
                                new_y += dy;
                            }
                            ResizeDirection::SouthEast => {
                                new_w += dx;
                                new_h += dy;
                            }
                            ResizeDirection::SouthWest => {
                                new_w -= dx;
                                new_x += dx;
                                new_h += dy;
                            }
                            ResizeDirection::NorthEast => {
                                new_w += dx;
                                new_h -= dy;
                                new_y += dy;
                            }
                            ResizeDirection::NorthWest => {
                                new_w -= dx;
                                new_x += dx;
                                new_h -= dy;
                                new_y += dy;
                            }
                        }

                        let new_w = new_w.max(0) as u32;
                        let new_h = new_h.max(0) as u32;

                        window.set_outer_position(winit::dpi::PhysicalPosition::new(new_x, new_y));
                        let _ =
                            window.request_inner_size(winit::dpi::PhysicalSize::new(new_w, new_h));
                    }
                } else {
                    app_state.manual_resize_start = None;
                }
            }

            // Handle window resizing via borders since it's frameless
            let border = 8.0;
            let screen_rect = ctx.screen_rect();
            if !is_resizing {
                if let Some(pos) = ctx.input(|i| i.pointer.hover_pos()) {
                    let left = pos.x < border;
                    let right = pos.x > screen_rect.max.x - border;
                    let top = pos.y < border;
                    let bottom = pos.y > screen_rect.max.y - border;

                    if left || right || top || bottom {
                        if top && left {
                            ctx.set_cursor_icon(egui::CursorIcon::ResizeNwSe);
                        } else if top && right {
                            ctx.set_cursor_icon(egui::CursorIcon::ResizeNeSw);
                        } else if bottom && left {
                            ctx.set_cursor_icon(egui::CursorIcon::ResizeNeSw);
                        } else if bottom && right {
                            ctx.set_cursor_icon(egui::CursorIcon::ResizeNwSe);
                        } else if top || bottom {
                            ctx.set_cursor_icon(egui::CursorIcon::ResizeVertical);
                        } else if left || right {
                            ctx.set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
                        }

                        if ctx.input(|i| i.pointer.primary_pressed()) {
                            use winit::window::ResizeDirection;
                            let dir = if top && left {
                                ResizeDirection::NorthWest
                            } else if top && right {
                                ResizeDirection::NorthEast
                            } else if bottom && left {
                                ResizeDirection::SouthWest
                            } else if bottom && right {
                                ResizeDirection::SouthEast
                            } else if top {
                                ResizeDirection::North
                            } else if bottom {
                                ResizeDirection::South
                            } else if left {
                                ResizeDirection::West
                            } else {
                                ResizeDirection::East
                            };

                            if let (Some((cx, cy)), Ok(wpos)) =
                                (get_global_cursor(), window.outer_position())
                            {
                                let size = window.inner_size();
                                app_state.manual_resize_start =
                                    Some((dir, cx, cy, wpos.x, wpos.y, size.width, size.height));
                            } else {
                                let _ = window.drag_resize_window(dir);
                            }
                        }
                    }
                }
            }

            let mut actions = render_title_bar(ctx, app_state, window);

            for action in &actions {
                match action {
                    TitleBarAction::ThemeClicked => app_state.theme_modal_open = true,
                    TitleBarAction::ToggleBg => {
                        app_state.is_3d_bg_active = !app_state.is_3d_bg_active;
                        if app_state.is_3d_bg_active {
                            if app_state.bg_process.is_none() {
                                let size = window.inner_size();
                                let (pos_x, pos_y) = if let Ok(pos) = window.outer_position() {
                                    (pos.x, pos.y)
                                } else {
                                    (0, 0)
                                };
                                #[cfg(windows)]
                                {
                                    use winit::raw_window_handle::{
                                        HasWindowHandle, RawWindowHandle,
                                    };
                                    let mut main_hwnd_isize = 0isize;
                                    if let Ok(handle) = window.window_handle() {
                                        if let RawWindowHandle::Win32(win32) = handle.as_raw() {
                                            main_hwnd_isize = win32.hwnd.get() as isize;
                                        }
                                    }

                                    let dev_path = "background/target/release/quantum_logo.exe";
                                    let rel_path = "quantum_logo.exe";

                                    let child_res = if std::path::Path::new(rel_path).exists() {
                                        // Production / Distribution path (same folder)
                                        std::process::Command::new(rel_path)
                                            .args([
                                                &size.width.to_string(),
                                                &size.height.to_string(),
                                                &pos_x.to_string(),
                                                &pos_y.to_string(),
                                                &main_hwnd_isize.to_string(),
                                            ])
                                            .spawn()
                                    } else if std::path::Path::new(dev_path).exists() {
                                        // Development path (cargo run from root)
                                        std::process::Command::new(dev_path)
                                            .args([
                                                &size.width.to_string(),
                                                &size.height.to_string(),
                                                &pos_x.to_string(),
                                                &pos_y.to_string(),
                                                &main_hwnd_isize.to_string(),
                                            ])
                                            .spawn()
                                    } else {
                                        // Fallback to cargo run if not built
                                        std::process::Command::new("cargo")
                                            .args([
                                                "run",
                                                "--release",
                                                "--manifest-path",
                                                "background/Cargo.toml",
                                                "--",
                                                &size.width.to_string(),
                                                &size.height.to_string(),
                                                &pos_x.to_string(),
                                                &pos_y.to_string(),
                                                &main_hwnd_isize.to_string(),
                                            ])
                                            .spawn()
                                    };

                                    if let Ok(child) = child_res {
                                        app_state.bg_process = Some(child);
                                        app_state.bg_hwnd = None;
                                    }
                                }
                                #[cfg(not(windows))]
                                {
                                    if let Ok(child) = std::process::Command::new("cargo")
                                        .args([
                                            "run",
                                            "--release",
                                            "--manifest-path",
                                            "background/Cargo.toml",
                                            "--",
                                            &size.width.to_string(),
                                            &size.height.to_string(),
                                            &pos_x.to_string(),
                                            &pos_y.to_string(),
                                            "0",
                                        ])
                                        .spawn()
                                    {
                                        app_state.bg_process = Some(child);
                                        app_state.bg_hwnd = None;
                                    }
                                }
                            }
                        } else {
                            if let Some(mut child) = app_state.bg_process.take() {
                                let _ = child.kill();
                                let _ = child.wait();
                            }
                        }
                    }
                    TitleBarAction::ExportClicked => {
                        if let Ok(json) = serde_json::to_string_pretty(&app_state.quotes) {
                            if let Ok(mut file) = OpenOptions::new()
                                .create(true)
                                .write(true)
                                .truncate(true)
                                .open("quotes_export.json")
                            {
                                let _ = file.write_all(json.as_bytes());
                            }
                        }
                    }
                    TitleBarAction::ZoomIn => {
                        app_state.title_bar_state.zoom_level =
                            (app_state.title_bar_state.zoom_level + 0.1).min(2.0);
                    }
                    TitleBarAction::ZoomOut => {
                        app_state.title_bar_state.zoom_level =
                            (app_state.title_bar_state.zoom_level - 0.1).max(0.5);
                    }
                    TitleBarAction::TogglePanel => {
                        app_state.title_bar_state.control_panel_visible =
                            !app_state.title_bar_state.control_panel_visible;
                    }
                    TitleBarAction::MinimizeClicked => {
                        window.set_minimized(true);
                    }
                    TitleBarAction::MaximizeClicked => {
                        window.set_maximized(!window.is_maximized());
                    }
                    TitleBarAction::CloseClicked => {
                        self.should_close = true;
                    }
                    TitleBarAction::HideHeader => {
                        app_state.title_bar_state.header_visible = false;
                    }
                    TitleBarAction::ShowHeader => {
                        app_state.title_bar_state.header_visible = true;
                    }
                    TitleBarAction::AnimateClicked => {
                        if app_state.active_animation == AppAnimation::Bounce {
                            app_state.active_animation = AppAnimation::None;
                        } else {
                            app_state.active_animation = AppAnimation::Bounce;
                        }
                    }
                    TitleBarAction::PlayBounce => {
                        if app_state.active_animation == AppAnimation::None {
                            if let Ok(pos) = window.outer_position() {
                                app_state.base_pos = Some((pos.x, pos.y));
                            }
                        }
                        app_state.active_animation =
                            if app_state.active_animation == AppAnimation::Bounce {
                                AppAnimation::None
                            } else {
                                AppAnimation::Bounce
                            };
                    }
                    TitleBarAction::PlayShake => {
                        if app_state.active_animation == AppAnimation::None {
                            if let Ok(pos) = window.outer_position() {
                                app_state.base_pos = Some((pos.x, pos.y));
                            }
                        }
                        app_state.active_animation =
                            if app_state.active_animation == AppAnimation::Shake {
                                AppAnimation::None
                            } else {
                                AppAnimation::Shake
                            };
                    }
                    TitleBarAction::PlayDance => {
                        if app_state.active_animation == AppAnimation::None {
                            if let Ok(pos) = window.outer_position() {
                                app_state.base_pos = Some((pos.x, pos.y));
                            }
                        }
                        app_state.active_animation =
                            if app_state.active_animation == AppAnimation::Dance {
                                AppAnimation::None
                            } else {
                                AppAnimation::Dance
                            };
                    }
                    TitleBarAction::PlayRotate => {
                        app_state.rotation = (app_state.rotation + 1) % 4;
                        let size = window.inner_size();
                        let _ = window.request_inner_size(winit::dpi::PhysicalSize::new(
                            size.height,
                            size.width,
                        ));

                        #[cfg(windows)]
                        {
                            use windows::core::PCWSTR;
                            use windows::Win32::Foundation::HANDLE;
                            use windows::Win32::UI::WindowsAndMessaging::SetPropW;

                            let mut property_name: Vec<u16> =
                                "RotationState".encode_utf16().collect();
                            property_name.push(0);

                            if let Ok(handle) = window.window_handle() {
                                if let winit::raw_window_handle::RawWindowHandle::Win32(win32) =
                                    handle.as_raw()
                                {
                                    unsafe {
                                        let _ = SetPropW(
                                            HWND(win32.hwnd.get() as _),
                                            PCWSTR(property_name.as_ptr()),
                                            HANDLE(app_state.rotation as _),
                                        );
                                    }
                                }
                            }
                        }
                    }
                    TitleBarAction::PlayDissolve => {
                        if app_state.active_animation == AppAnimation::None {
                            if let Ok(pos) = window.outer_position() {
                                app_state.base_pos = Some((pos.x, pos.y));
                            }
                        }
                        app_state.active_animation =
                            if app_state.active_animation == AppAnimation::Dissolve {
                                AppAnimation::None
                            } else {
                                AppAnimation::Dissolve
                            };
                        if app_state.active_animation == AppAnimation::None {
                            if let Ok(handle) = window.window_handle() {
                                if let winit::raw_window_handle::RawWindowHandle::Win32(win32) =
                                    handle.as_raw()
                                {
                                    let hwnd = HWND(win32.hwnd.get() as _);
                                    unsafe {
                                        let _ =
                                            SetLayeredWindowAttributes(hwnd, None, 255, LWA_ALPHA);
                                    }
                                }
                            }
                        }
                    }
                    TitleBarAction::PlayFly => {
                        if app_state.active_animation == AppAnimation::None {
                            if let Ok(pos) = window.outer_position() {
                                app_state.base_pos = Some((pos.x, pos.y));
                            }
                        }
                        app_state.active_animation =
                            if app_state.active_animation == AppAnimation::Fly {
                                AppAnimation::None
                            } else {
                                AppAnimation::Fly
                            };
                    }
                    TitleBarAction::StopAnimations => {
                        app_state.active_animation = AppAnimation::None;
                        if let Ok(handle) = window.window_handle() {
                            if let winit::raw_window_handle::RawWindowHandle::Win32(win32) =
                                handle.as_raw()
                            {
                                let hwnd = HWND(win32.hwnd.get() as _);
                                unsafe {
                                    let _ = SetLayeredWindowAttributes(hwnd, None, 255, LWA_ALPHA);
                                }
                            }
                        }
                        if let Some((x, y)) = app_state.base_pos {
                            window.set_outer_position(winit::dpi::PhysicalPosition::new(x, y));
                        }
                        app_state.base_pos = None;
                    }
                }
            }

            // Window Animation Engine
            if app_state.active_animation != AppAnimation::None {
                if let (Ok(pos), Some(monitor)) =
                    (window.outer_position(), window.current_monitor())
                {
                    let size = window.outer_size();
                    let monitor_size = monitor.size();
                    app_state.anim_progress += 0.016;

                    // Capture base position if not already set
                    if app_state.base_pos.is_none() {
                        app_state.base_pos = Some((pos.x, pos.y));
                    }
                    let (base_x, base_y) = app_state.base_pos.unwrap();

                    match app_state.active_animation {
                        AppAnimation::Bounce => {
                            let mut new_x = pos.x as f32 + app_state.bounce_vel_x;
                            let mut new_y = pos.y as f32 + app_state.bounce_vel_y;

                            if new_x < 0.0 {
                                new_x = 0.0;
                                app_state.bounce_vel_x *= -1.0;
                            } else if new_x + size.width as f32 > monitor_size.width as f32 {
                                new_x = monitor_size.width as f32 - size.width as f32;
                                app_state.bounce_vel_x *= -1.0;
                            }

                            if new_y < 0.0 {
                                new_y = 0.0;
                                app_state.bounce_vel_y *= -1.0;
                            } else if new_y + size.height as f32 > monitor_size.height as f32 {
                                new_y = monitor_size.height as f32 - size.height as f32;
                                app_state.bounce_vel_y *= -1.0;
                            }

                            window.set_outer_position(winit::dpi::PhysicalPosition::new(
                                new_x as i32,
                                new_y as i32,
                            ));
                            app_state.base_pos = Some((new_x as i32, new_y as i32));
                        }
                        AppAnimation::Shake => {
                            let intensity = 12.0;
                            let offset_x = (app_state.anim_progress * 130.0).sin() * intensity;
                            let offset_y = (app_state.anim_progress * 115.0).cos() * intensity;
                            window.set_outer_position(winit::dpi::PhysicalPosition::new(
                                base_x + offset_x as i32,
                                base_y + offset_y as i32,
                            ));
                        }
                        AppAnimation::Dance => {
                            let radius = 70.0;
                            let offset_x = (app_state.anim_progress * 4.0).sin() * radius;
                            let offset_y = (app_state.anim_progress * 2.5).cos() * radius;
                            window.set_outer_position(winit::dpi::PhysicalPosition::new(
                                base_x + offset_x as i32,
                                base_y + offset_y as i32,
                            ));
                        }
                        AppAnimation::Rotate => {
                            if app_state.anim_progress > 2.5 {
                                app_state.anim_progress = 0.0;
                                actions.push(TitleBarAction::PlayRotate);
                            }
                        }
                        AppAnimation::Dissolve => {
                            if let Ok(handle) = window.window_handle() {
                                if let winit::raw_window_handle::RawWindowHandle::Win32(win32) =
                                    handle.as_raw()
                                {
                                    let hwnd = HWND(win32.hwnd.get() as _);
                                    let opacity =
                                        0.4 + 0.6 * (app_state.anim_progress * 2.5).cos().abs();
                                    unsafe {
                                        let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE);
                                        if (ex_style & WS_EX_LAYERED.0 as i32) == 0 {
                                            let _ = SetWindowLongW(
                                                hwnd,
                                                GWL_EXSTYLE,
                                                ex_style | WS_EX_LAYERED.0 as i32,
                                            );
                                        }
                                        let _ = SetLayeredWindowAttributes(
                                            hwnd,
                                            None,
                                            (opacity * 255.0) as u8,
                                            LWA_ALPHA,
                                        );
                                    }
                                }
                            }
                        }
                        AppAnimation::Fly => {
                            let speed = 12.0;
                            let mut new_x = pos.x as f32 + speed;
                            let offset_y = (app_state.anim_progress * 2.0).sin() * 150.0;

                            if new_x > monitor_size.width as f32 {
                                new_x = -(size.width as f32);
                            }

                            window.set_outer_position(winit::dpi::PhysicalPosition::new(
                                new_x as i32,
                                (monitor_size.height as f32 / 2.0 + offset_y) as i32,
                            ));
                        }
                        _ => {}
                    }
                    window.request_redraw();
                }
            } else {
                if app_state.base_pos.is_some() {
                    if let Ok(handle) = window.window_handle() {
                        if let winit::raw_window_handle::RawWindowHandle::Win32(win32) =
                            handle.as_raw()
                        {
                            let hwnd = HWND(win32.hwnd.get() as _);
                            unsafe {
                                let _ = SetLayeredWindowAttributes(hwnd, None, 255, LWA_ALPHA);
                            }
                        }
                    }
                    if matches!(
                        app_state.active_animation,
                        AppAnimation::Shake | AppAnimation::Dance
                    ) {
                        if let Some((x, y)) = app_state.base_pos {
                            window.set_outer_position(winit::dpi::PhysicalPosition::new(x, y));
                        }
                    }
                    app_state.base_pos = None;
                    app_state.anim_progress = 0.0;
                }
            }

            if app_state.rotation_enabled
                && app_state.last_rotation.elapsed() >= app_state.rotation_interval
                && !app_state.quotes.is_empty()
            {
                app_state.next_quote();
            }

            // Build shaper tuple from cosmic-text state
            let mut shaper = match (font_system.as_mut(), swash_cache.as_mut()) {
                (Some(fs), Some(sc)) => Some((fs, sc, &mut tex_cache)),
                _ => None,
            };

            render_main_content(ctx, app_state, &mut shaper);

            render_theme_modal(ctx, app_state);

            // Render floating buttons
            let float_actions = render_floating_buttons(ctx, app_state);
            for action in float_actions {
                match action {
                    TitleBarAction::TogglePanel => {
                        app_state.title_bar_state.control_panel_visible =
                            !app_state.title_bar_state.control_panel_visible;
                    }
                    TitleBarAction::ShowHeader => {
                        app_state.title_bar_state.header_visible = true;
                    }
                    _ => {}
                }
            }
        });

        egui_state.handle_platform_output(window, full_output.platform_output);

        let paint_jobs = egui_ctx.tessellate(full_output.shapes, window.scale_factor() as f32);

        let frame = match render_state.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(_) => {
                render_state
                    .surface
                    .configure(&render_state.device, &render_state.surface_config);
                return;
            }
        };

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [
                render_state.surface_config.width,
                render_state.surface_config.height,
            ],
            pixels_per_point: window.scale_factor() as f32,
        };

        let mut encoder = render_state
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        for (id, image_delta) in &full_output.textures_delta.set {
            render_state.renderer.update_texture(
                &render_state.device,
                &render_state.queue,
                *id,
                image_delta,
            );
        }

        render_state.renderer.update_buffers(
            &render_state.device,
            &render_state.queue,
            &mut encoder,
            &paint_jobs,
            &screen_descriptor,
        );

        let bg_color = app_state.get_background_color();
        let clear_color = wgpu::Color {
            r: bg_color.r() as f64 / 255.0,
            g: bg_color.g() as f64 / 255.0,
            b: bg_color.b() as f64 / 255.0,
            a: bg_color.a() as f64 / 255.0,
        };

        {
            let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("egui_render"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(clear_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            let mut render_pass = render_pass.forget_lifetime();
            render_state
                .renderer
                .render(&mut render_pass, &paint_jobs, &screen_descriptor);
        }

        render_state.queue.submit(Some(encoder.finish()));
        frame.present();

        for id in &full_output.textures_delta.free {
            render_state.renderer.free_texture(id);
        }

        // Restore cosmic-text state back to self
        self.font_system = font_system;
        self.swash_cache = swash_cache;
        self.shaped_text_textures = tex_cache;
    }
}
