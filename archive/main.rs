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

use egui::epaint::ClippedShape;
use egui::Context;
use egui::FontId;
use egui::{Color32, Frame, RichText, Rounding, Sense, Stroke, TopBottomPanel, Vec2};
use egui::{Pos2, Rect, Shape};

#[cfg(windows)]
use windows::Win32::Foundation::HWND;
#[cfg(windows)]
use windows::Win32::UI::WindowsAndMessaging::{
    GetWindowLongW, SetLayeredWindowAttributes, SetPropW, SetWindowLongW, SetWindowPos,
    GWL_EXSTYLE, HWND_TOPMOST, LWA_ALPHA, SWP_NOMOVE, SWP_NOSIZE, SWP_SHOWWINDOW, WS_EX_LAYERED,
};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// =============================================================================
// CONSTANTS
// =============================================================================

const TITLE_BAR_HEIGHT: f32 = 26.0;

// ── DEEP VOID PALETTE ─────────────────────────────────
const BG_GLASS: Color32 = Color32::TRANSPARENT;

// ── QUANTUM NEON ACCENTS ──────────────────────────────
const NEON_CYAN: Color32 = Color32::from_rgb(0, 255, 220);
const NEON_PLASMA: Color32 = Color32::from_rgb(180, 0, 255);
const NEON_SOLAR: Color32 = Color32::from_rgb(255, 160, 0);
const NEON_LIME: Color32 = Color32::from_rgb(80, 255, 120);
const NEON_ROSE: Color32 = Color32::from_rgb(255, 40, 120);

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quote {
    pub main_text: String,
    pub sub_text: String,
    #[serde(default)]
    pub is_hidden: bool,
}

impl Default for Quote {
    fn default() -> Self {
        Self {
            main_text: "Focus on your goals - Success awaits!".to_string(),
            sub_text: "Keep pushing - You're doing great!".to_string(),
            is_hidden: false,
        }
    }
}

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
                Color32::from_rgb(2, 4, 16),
                Color32::from_rgb(30, 0, 80),
                Color32::from_rgb(0, 60, 120),
                Color32::from_rgb(0, 200, 180),
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextStyleConfig {
    pub main_text_size: f32,
    pub sub_text_size: f32,
    pub main_text_color: Color32,
    pub sub_text_color: Color32,
    #[serde(default = "default_panel_text_color")]
    pub panel_text_color: Color32,
    pub main_line_gap: f32,
    pub sub_line_gap: f32,
    pub between_gap: f32,
}

fn default_panel_text_color() -> Color32 {
    Color32::WHITE
}

impl Default for TextStyleConfig {
    fn default() -> Self {
        Self {
            main_text_size: 24.0,
            sub_text_size: 14.0,
            main_text_color: Color32::WHITE,
            sub_text_color: Color32::from_rgba_unmultiplied(255, 255, 255, 200),
            panel_text_color: Color32::WHITE,
            main_line_gap: 1.6,
            sub_line_gap: 1.6,
            between_gap: 15.0,
        }
    }
}

// =============================================================================
// TITLE BAR ICON DEFINITIONS
// =============================================================================

#[derive(Debug, Clone)]
pub struct TitleBarIcon {
    pub symbol: &'static str,
    pub tooltip: &'static str,
    pub width: f32,
    pub font_size: f32,
}

impl TitleBarIcon {
    pub const fn new(symbol: &'static str, tooltip: &'static str, width: f32, font_size: f32) -> Self {
        Self { symbol, tooltip, width, font_size }
    }
}

pub mod icons {
    use super::TitleBarIcon;
    pub const APP_ICON: TitleBarIcon = TitleBarIcon::new("\u{f135}", "Daily Motivation", 20.0, 24.0);
    pub const THEME: TitleBarIcon = TitleBarIcon::new("\u{eb5c}", "Change Theme", 20.0, 12.0);
    pub const TOGGLE_BG: TitleBarIcon = TitleBarIcon::new("\u{f110}", "Toggle 3D Background", 20.0, 16.0);
    pub const EXPORT: TitleBarIcon = TitleBarIcon::new("\u{f0207}", "Export Quotes", 20.0, 13.2);
    pub const ZOOM_IN: TitleBarIcon = TitleBarIcon::new("\u{f120d}", "Zoom In", 20.0, 16.8);
    pub const ZOOM_OUT: TitleBarIcon = TitleBarIcon::new("\u{f06ec}", "Zoom Out", 20.0, 16.8);
    pub const TOGGLE_PANEL: TitleBarIcon = TitleBarIcon::new("\u{f0c9}", "Toggle Panel", 20.0, 24.0);
    pub const MINIMIZE: TitleBarIcon = TitleBarIcon::new("\u{f2d1}", "Minimize", 20.0, 11.2);
    pub const MAXIMIZE: TitleBarIcon = TitleBarIcon::new("\u{f2d0}", "Maximize", 20.0, 10.0);
    pub const CLOSE: TitleBarIcon = TitleBarIcon::new("\u{f110a}", "Close", 20.0, 13.2);
    pub const HIDE_HEADER: TitleBarIcon = TitleBarIcon::new("\u{f102}", "Hide Header", 20.0, 17.5);
    pub const SHOW_HEADER: TitleBarIcon = TitleBarIcon::new("\u{f103}", "Show Header", 20.0, 24.0);
    pub const ROTATE: TitleBarIcon = TitleBarIcon::new("\u{f01e}", "Rotate Window", 20.0, 16.0);
    pub const ANIMATE: TitleBarIcon = TitleBarIcon::new("\u{f04b}", "Animate Window", 20.0, 16.0);
    pub const ANIM_BOUNCE: TitleBarIcon = TitleBarIcon::new("\u{f0025}", "Bounce Animation", 20.0, 16.0);
    pub const ANIM_SHAKE: TitleBarIcon = TitleBarIcon::new("\u{f067a}", "Shake Animation", 20.0, 16.0);
    pub const ANIM_DANCE: TitleBarIcon = TitleBarIcon::new("\u{f00d2}", "Dance Animation", 20.0, 16.0);
    pub const ANIM_ROTATE: TitleBarIcon = TitleBarIcon::new("\u{f01e}", "Rotate Animation", 20.0, 16.0);
    pub const ANIM_DISSOLVE: TitleBarIcon = TitleBarIcon::new("\u{f0376}", "Dissolve Animation", 20.0, 16.0);
    pub const ANIM_FLY: TitleBarIcon = TitleBarIcon::new("\u{f02eb}", "Fly Animation", 20.0, 16.0);
}

// =============================================================================
// UI STATE
// =============================================================================

#[derive(Debug)]
pub struct TitleBarState {
    pub theme_btn_hovered: bool,
    pub toggle_bg_btn_hovered: bool,
    pub export_btn_hovered: bool,
    pub zoom_out_btn_hovered: bool,
    pub zoom_in_btn_hovered: bool,
    pub toggle_panel_btn_hovered: bool,
    pub minimize_btn_hovered: bool,
    pub maximize_btn_hovered: bool,
    pub close_btn_hovered: bool,
    pub control_panel_visible: bool,
    pub header_visible: bool,
    pub zoom_level: f32,
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TitleBarAction {
    ThemeClicked, ToggleBg, ExportClicked, ZoomIn, ZoomOut, TogglePanel,
    MinimizeClicked, MaximizeClicked, CloseClicked, ShowHeader, HideHeader,
    AnimateClicked, PlayBounce, PlayShake, PlayDance, PlayRotate, PlayDissolve,
    PlayFly, StopAnimations,
}

// =============================================================================
// ANIMATION TYPES
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub enum AppAnimation {
    #[default] None, Bounce, Shake, Dance, Rotate, Dissolve, Fly,
}

// =============================================================================
// PERSISTENCE CONFIGURATION
// =============================================================================

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
            let _ = serde_json::to_writer_pretty(file, self);
        }
    }
}

// =============================================================================
// MAIN APPLICATION STATE
// =============================================================================

#[derive(Debug)]
pub struct AppState {
    pub title_bar_state: TitleBarState,
    pub quotes: Vec<Quote>,
    pub current_quote_index: usize,
    pub rotation_interval: Duration,
    pub last_rotation: Instant,
    pub rotation_enabled: bool,
    pub interval_secs: u64,
    pub theme: ThemeConfig,
    pub theme_modal_open: bool,
    pub text_style: TextStyleConfig,
    pub main_text_input: String,
    pub sub_text_input: String,
    pub subtitle_editing: bool,
    pub subtitle_edit_buffer: String,
    pub confirm_clear_pending: bool,
    pub is_3d_bg_active: bool,
    pub bg_process: Option<std::process::Child>,
    pub bg_hwnd: Option<isize>,
    pub show_main_color_picker: bool,
    pub show_sub_color_picker: bool,
    pub show_panel_color_picker: bool,
    pub running: bool,
    pub last_interaction: Instant,
    pub manual_resize_start: Option<(winit::window::ResizeDirection, i32, i32, i32, i32, u32, u32)>,
    pub rotation: u8,
    pub target_rotation_angle: f32,
    pub current_rotation_angle: f32,
    pub current_scale: f32,
    pub active_animation: AppAnimation,
    pub anim_progress: f32,
    pub bounce_vel_x: f32,
    pub bounce_vel_y: f32,
    pub base_pos: Option<(i32, i32)>,
    // NEW: drag-reorder state
    pub drag_reorder_from: Option<usize>,
}

impl Default for AppState {
    fn default() -> Self {
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
                show_panel_color_picker: false,
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
                target_rotation_angle: 0.0,
                current_rotation_angle: 0.0,
                current_scale: 1.0,
                active_animation: AppAnimation::None,
                anim_progress: 0.0,
                bounce_vel_x: 5.0,
                bounce_vel_y: 4.0,
                base_pos: None,
                drag_reorder_from: None,
            }
        } else {
            Self {
                title_bar_state: TitleBarState::default(),
                quotes: vec![
                    Quote { main_text: "এখনই কাজে মনোযোগ দাও - ফোকাস তোমার শক্তি".to_string(), sub_text: "Keep pushing - You're doing great! 🌟".to_string(), is_hidden: false },
                    Quote { main_text: "প্রতিটি মুহূর্ত গুরুত্বপূর্ণ - কাজ চালিয়ে যাও".to_string(), sub_text: "Keep pushing - You're doing great! 🌟".to_string(), is_hidden: false },
                    Quote { main_text: "সফলতা ধৈর্যের ফল - হার মানিও না".to_string(), sub_text: "Keep pushing - You're doing great! 🌟".to_string(), is_hidden: false },
                    Quote { main_text: "Focus on the work - Success is near".to_string(), sub_text: "Keep pushing - You're doing great! 🌟".to_string(), is_hidden: false },
                    Quote { main_text: "Stay disciplined - Great things take time".to_string(), sub_text: "Keep pushing - You're doing great! 🌟".to_string(), is_hidden: false },
                    Quote { main_text: "তুমি পারবে - শুধু চেষ্টা চালিয়ে যাও".to_string(), sub_text: "Keep pushing - You're doing great! 🌟".to_string(), is_hidden: false },
                    Quote { main_text: "Dreams need action - Start now".to_string(), sub_text: "Keep pushing - You're doing great! 🌟".to_string(), is_hidden: false },
                    Quote { main_text: "প্রতিদিন একটু এগিয়ে যাও - লক্ষ্য কাছে".to_string(), sub_text: "Keep pushing - You're doing great! 🌟".to_string(), is_hidden: false },
                    Quote { main_text: "Consistency beats talent - Keep going".to_string(), sub_text: "Keep pushing - You're doing great! 🌟".to_string(), is_hidden: false },
                    Quote { main_text: "বিশ্রাম নাও কিন্তু হাল ছাড়ো না".to_string(), sub_text: "Keep pushing - You're doing great! 🌟".to_string(), is_hidden: false },
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
                show_panel_color_picker: false,
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
                target_rotation_angle: 0.0,
                current_rotation_angle: 0.0,
                current_scale: 1.0,
                active_animation: AppAnimation::None,
                anim_progress: 0.0,
                bounce_vel_x: 5.0,
                bounce_vel_y: 4.0,
                base_pos: None,
                drag_reorder_from: None,
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
    pub fn save(&self) {
        let config = AppConfig {
            quotes: self.quotes.clone(),
            interval_secs: self.interval_secs,
            theme: self.theme.clone(),
            text_style: self.text_style.clone(),
        };
        config.save();
    }

    pub fn current_quote(&self) -> Option<&Quote> {
        self.quotes.get(self.current_quote_index)
    }

    /// Get next visible (non-hidden) quote index starting from current+1
    fn next_visible_index(&self, from: usize) -> Option<usize> {
        let len = self.quotes.len();
        if len == 0 { return None; }
        for i in 1..=len {
            let idx = (from + i) % len;
            if !self.quotes[idx].is_hidden {
                return Some(idx);
            }
        }
        None // all hidden
    }

    pub fn next_quote(&mut self) {
        if !self.quotes.is_empty() {
            // Skip hidden quotes
            if let Some(idx) = self.next_visible_index(self.current_quote_index) {
                self.current_quote_index = idx;
            }
            self.last_rotation = Instant::now();
        }
    }

    pub fn prev_quote(&mut self) {
        if !self.quotes.is_empty() {
            let len = self.quotes.len();
            // Walk backwards to find previous visible
            for i in 1..=len {
                let idx = (self.current_quote_index + len - i) % len;
                if !self.quotes[idx].is_hidden {
                    self.current_quote_index = idx;
                    break;
                }
            }
            self.last_rotation = Instant::now();
        }
    }

    pub fn add_quote(&mut self, main: String, sub: String) {
        let sub = if sub.is_empty() {
            "Keep pushing - You're doing great! 🌟".to_string()
        } else { sub };
        self.quotes.push(Quote { main_text: main, sub_text: sub, is_hidden: false });
        self.current_quote_index = self.quotes.len() - 1;
        self.save();
    }

    pub fn delete_quote(&mut self, index: usize) {
        if index < self.quotes.len() {
            self.quotes.remove(index);
            if self.current_quote_index >= self.quotes.len() && !self.quotes.is_empty() {
                self.current_quote_index = self.quotes.len() - 1;
            }
            self.save();
        }
    }

    /// Move quote at `from` to position `to`
    pub fn move_quote(&mut self, from: usize, to: usize) {
        let len = self.quotes.len();
        if from == to || from >= len || to >= len { return; }
        let quote = self.quotes.remove(from);
        self.quotes.insert(to, quote);
        // Adjust current index
        if self.current_quote_index == from {
            self.current_quote_index = to;
        } else if from < to && self.current_quote_index > from && self.current_quote_index <= to {
            self.current_quote_index -= 1;
        } else if from > to && self.current_quote_index >= to && self.current_quote_index < from {
            self.current_quote_index += 1;
        }
        self.save();
    }

    pub fn get_background_color(&self) -> Color32 {
        if self.is_3d_bg_active { return Color32::TRANSPARENT; }
        if self.theme.mode == ThemeMode::Solid { return self.theme.solid_color; }
        self.theme.gradient_colors.first().copied().unwrap_or(CANVAS_BG)
    }

    /// Get the current display quote — skipping hidden ones
    pub fn current_display_quote(&self) -> Option<&Quote> {
        let q = self.quotes.get(self.current_quote_index)?;
        if !q.is_hidden { Some(q) } else { None }
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
    if response.hovered() { ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand); }
    let is_hovered = response.hovered();
    if is_hovered {
        let glow_rect = rect.expand(2.0);
        ui.painter().rect_filled(glow_rect, Rounding::same(8.0), NEON_CYAN.gamma_multiply(0.12));
        ui.painter().rect_stroke(glow_rect, Rounding::same(8.0), Stroke::new(1.0, NEON_CYAN.gamma_multiply(0.47)));
    }
    let bg = if is_hovered { NEON_CYAN.gamma_multiply(0.11) } else { BG_GLASS };
    ui.painter().rect_filled(rect, Rounding::same(6.0), bg);
    let top_line = [egui::pos2(rect.left() + 4.0, rect.top() + 1.0), egui::pos2(rect.right() - 4.0, rect.top() + 1.0)];
    ui.painter().line_segment(top_line, Stroke::new(1.0, if is_hovered { NEON_CYAN.gamma_multiply(0.7) } else { Color32::from_rgba_premultiplied(255, 255, 255, 25) }));
    let icon_color = if is_hovered { NEON_CYAN } else { fg_color };
    ui.painter().text(rect.center(), egui::Align2::CENTER_CENTER, icon.symbol, FontId::proportional(icon.font_size), icon_color);
    response
}

pub fn draw_text_button(ui: &mut egui::Ui, text: &str, bg_color: Color32, width: f32, height: f32) -> egui::Response {
    let size = Vec2::new(width, height);
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    if response.hovered() { ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand); }
    let is_hovered = response.hovered();
    let is_clicked = response.is_pointer_button_down_on();
    if is_hovered {
        ui.painter().rect_filled(rect.expand(3.0), Rounding::same(8.0), Color32::from_rgba_unmultiplied(bg_color.r(), bg_color.g(), bg_color.b(), 18));
    }
    let bg = if is_clicked { bg_color.linear_multiply(1.4) } else if is_hovered { bg_color.linear_multiply(1.15) } else { bg_color.linear_multiply(0.75) };
    ui.painter().rect_filled(rect, Rounding::same(6.0), bg);
    ui.painter().line_segment([egui::pos2(rect.left() + 6.0, rect.top() + 1.0), egui::pos2(rect.right() - 6.0, rect.top() + 1.0)], Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, if is_hovered { 60 } else { 20 })));
    ui.painter().rect_stroke(rect, Rounding::same(6.0), Stroke::new(1.0, if is_hovered { Color32::from_rgba_unmultiplied(bg_color.r(), bg_color.g(), bg_color.b(), 200) } else { Color32::from_rgba_unmultiplied(bg_color.r(), bg_color.g(), bg_color.b(), 80) }));
    let center = rect.center();
    let font_id = FontId::proportional(11.5);
    let shadow = Color32::from_black_alpha(130);
    let offsets: [Vec2; 8] = [Vec2::new(0.5, 0.0), Vec2::new(-0.5, 0.0), Vec2::new(0.0, 0.5), Vec2::new(0.0, -0.5), Vec2::new(0.5, 0.5), Vec2::new(-0.5, 0.5), Vec2::new(0.5, -0.5), Vec2::new(-0.5, -0.5)];
    for offset in offsets { ui.painter().text(center + offset, egui::Align2::CENTER_CENTER, text, font_id.clone(), shadow); }
    ui.painter().text(center, egui::Align2::CENTER_CENTER, text, font_id, Color32::WHITE);
    response
}

fn label_with_glow(ui: &mut egui::Ui, text: &str, main_color: Color32, size: f32, shadow_or_glow_color: Color32, align: egui::Align2) -> egui::Response {
    let font_id = FontId::proportional(size);
    let approx_w = (text.len() as f32 * size * 0.55).max(20.0) + 2.0;
    let approx_h = size * 1.8 + 2.0;
    let allocate_size = Vec2::new(approx_w, approx_h);
    let (rect, response) = ui.allocate_exact_size(allocate_size, Sense::hover());
    let pos = match align {
        egui::Align2::LEFT_CENTER => rect.left_center() + Vec2::new(0.0, -1.0),
        egui::Align2::RIGHT_CENTER => rect.right_center() - Vec2::new(0.0, 1.0),
        _ => rect.center() - Vec2::new(0.0, 1.0),
    };
    let offsets: [Vec2; 8] = [Vec2::new(0.5, 0.0), Vec2::new(-0.5, 0.0), Vec2::new(0.0, 0.5), Vec2::new(0.0, -0.5), Vec2::new(0.5, 0.5), Vec2::new(-0.5, 0.5), Vec2::new(0.5, -0.5), Vec2::new(-0.5, -0.5)];
    let reduced_glow = shadow_or_glow_color.gamma_multiply(0.25);
    for offset in offsets { ui.painter().text(pos + offset, align, text, font_id.clone(), reduced_glow); }
    ui.painter().text(pos, align, text, font_id, main_color);
    response
}

// =============================================================================
// TITLE BAR RENDERER
// =============================================================================

pub fn render_title_bar(ctx: &Context, state: &mut AppState, window: &Window) -> Vec<TitleBarAction> {
    if !state.title_bar_state.header_visible { return Vec::new(); }
    let mut actions = Vec::new();
    let titlebar_bg = Color32::from_black_alpha(26);
    TopBottomPanel::top("title_bar")
        .exact_height(TITLE_BAR_HEIGHT)
        .frame(Frame::none().fill(titlebar_bg))
        .show(ctx, |ui| {
            let rect = ui.max_rect();
            ui.painter().line_segment([rect.left_top(), rect.right_top()], Stroke::new(1.5, TITLEBAR_FG.gamma_multiply(0.78)));
            ui.painter().line_segment([egui::pos2(rect.left(), rect.top() + 3.0), egui::pos2(rect.right(), rect.top() + 3.0)], Stroke::new(0.5, TITLEBAR_FG.gamma_multiply(0.15)));
            let b = 8.0;
            let stroke = Stroke::new(1.5, TITLEBAR_FG.gamma_multiply(0.63));
            ui.painter().line_segment([egui::pos2(rect.left(), rect.top()), egui::pos2(rect.left() + b, rect.top())], stroke);
            ui.painter().line_segment([egui::pos2(rect.left(), rect.top()), egui::pos2(rect.left(), rect.bottom())], stroke);
            ui.painter().line_segment([egui::pos2(rect.right() - b, rect.top()), egui::pos2(rect.right(), rect.top())], stroke);
            ui.painter().line_segment([egui::pos2(rect.right(), rect.top()), egui::pos2(rect.right(), rect.bottom())], stroke);
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                ui.spacing_mut().item_spacing = Vec2::new(4.0, 0.0);
                ui.add_space(12.0);
                ui.label(RichText::new(icons::APP_ICON.symbol).size(15.0).color(TITLEBAR_FG));
                ui.label(RichText::new("DAILY  MOTIVATION").color(TITLEBAR_FG).strong().size(12.0));
                ui.add_space(4.0);
                let (br, _) = ui.allocate_exact_size(Vec2::new(38.0, 14.0), Sense::hover());
                ui.painter().rect_filled(br, Rounding::same(3.0), TITLEBAR_FG.gamma_multiply(0.08));
                ui.painter().rect_stroke(br, Rounding::same(3.0), Stroke::new(0.5, TITLEBAR_FG.gamma_multiply(0.31)));
                ui.painter().text(br.center(), egui::Align2::CENTER_CENTER, "v∞.0", FontId::proportional(8.5), TITLEBAR_FG.gamma_multiply(0.7));
                ui.add_space(8.0);
                if !state.quotes.is_empty() {
                    let visible_count = state.quotes.iter().filter(|q| !q.is_hidden).count();
                    ui.label(RichText::new(format!("[ {}/{} ]", state.current_quote_index + 1, state.quotes.len())).color(NEON_LIME.gamma_multiply(0.7)).size(10.5));
                    if visible_count < state.quotes.len() {
                        ui.label(RichText::new(format!("({} hidden)", state.quotes.len() - visible_count)).color(NEON_SOLAR.gamma_multiply(0.6)).size(9.5));
                    }
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(3.0, 0.0);
                    ui.add_space(6.0);
                    let btns = [
                        (&icons::CLOSE, NEON_ROSE, TitleBarAction::CloseClicked),
                        (&icons::MAXIMIZE, Color32::WHITE, TitleBarAction::MaximizeClicked),
                        (&icons::MINIMIZE, Color32::WHITE, TitleBarAction::MinimizeClicked),
                    ];
                    for (icon, color, action) in btns {
                        if draw_icon_button(ui, icon, Color32::TRANSPARENT, color, false).clicked() { actions.push(action); }
                    }
                    if draw_icon_button(ui, &icons::HIDE_HEADER, Color32::TRANSPARENT, Color32::WHITE, false).clicked() { actions.push(TitleBarAction::HideHeader); }
                    ui.add_space(8.0);
                    let anim_btns = [
                        (&icons::ANIM_FLY, TitleBarAction::PlayFly, AppAnimation::Fly),
                        (&icons::ANIM_DISSOLVE, TitleBarAction::PlayDissolve, AppAnimation::Dissolve),
                        (&icons::ANIM_ROTATE, TitleBarAction::PlayRotate, AppAnimation::Rotate),
                        (&icons::ANIM_DANCE, TitleBarAction::PlayDance, AppAnimation::Dance),
                        (&icons::ANIM_SHAKE, TitleBarAction::PlayShake, AppAnimation::Shake),
                        (&icons::ANIM_BOUNCE, TitleBarAction::PlayBounce, AppAnimation::Bounce),
                    ];
                    for (icon, action, anim_type) in anim_btns {
                        let active = state.active_animation == anim_type;
                        let color = if active { NEON_LIME } else { Color32::WHITE };
                        if draw_icon_button(ui, icon, Color32::TRANSPARENT, color, active).clicked() { actions.push(action); }
                    }
                    ui.add_space(8.0);
                    let bg_color = if state.is_3d_bg_active { NEON_CYAN } else { Color32::from_rgba_premultiplied(255, 255, 255, 150) };
                    if draw_icon_button(ui, &icons::TOGGLE_BG, Color32::TRANSPARENT, bg_color, false).clicked() { actions.push(TitleBarAction::ToggleBg); }
                    ui.add_space(8.0);
                    if draw_icon_button(ui, &icons::ZOOM_IN, Color32::TRANSPARENT, Color32::WHITE, false).clicked() { actions.push(TitleBarAction::ZoomIn); }
                    if draw_icon_button(ui, &icons::ZOOM_OUT, Color32::TRANSPARENT, Color32::WHITE, false).clicked() { actions.push(TitleBarAction::ZoomOut); }
                    ui.add_space(8.0);
                    if draw_icon_button(ui, &icons::EXPORT, Color32::TRANSPARENT, Color32::WHITE, false).clicked() { actions.push(TitleBarAction::ExportClicked); }
                    if draw_icon_button(ui, &icons::THEME, Color32::TRANSPARENT, Color32::WHITE, false).clicked() { actions.push(TitleBarAction::ThemeClicked); }
                    let drag_avail = ui.available_width();
                    if drag_avail > 0.0 {
                        let (_, resp) = ui.allocate_exact_size(Vec2::new(drag_avail, TITLE_BAR_HEIGHT), Sense::drag());
                        if resp.drag_started() { let _ = window.drag_window(); }
                    }
                });
            });
            actions
        }).inner
}

fn render_floating_buttons(ctx: &Context, state: &mut AppState) -> Vec<TitleBarAction> {
    let mut actions = Vec::new();
    let elapsed = state.last_interaction.elapsed().as_secs_f32();
    let opacity = if elapsed > 5.0 { 1.0 - ((elapsed - 5.0) / 0.5).min(1.0) } else { 1.0 };
    if opacity <= 0.0 { return actions; }
    let screen_rect = ctx.screen_rect();
    let pos = egui::pos2(screen_rect.right() - 3.0, TITLE_BAR_HEIGHT + 2.0);
    egui::Area::new(egui::Id::new("floating_buttons"))
        .fixed_pos(pos).pivot(egui::Align2::RIGHT_TOP)
        .order(egui::Order::Foreground).interactable(opacity > 0.0)
        .show(ctx, |ui| {
            if opacity < 1.0 && opacity > 0.0 { ui.ctx().request_repaint(); }
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing = Vec2::new(0.0, 8.0);
                let (bg, fg) = if state.title_bar_state.control_panel_visible { (BTN_ACTIVE_BG, BTN_ACTIVE_FG) } else { (BTN_NORMAL_BG, Color32::WHITE) };
                let bg = bg.linear_multiply(opacity);
                let fg = fg.linear_multiply(opacity);
                let btn_icon = if state.title_bar_state.control_panel_visible { &icons::TOGGLE_PANEL } else { &icons::CLOSE };
                let btn_tooltip = if state.title_bar_state.control_panel_visible { "Hide Panel" } else { "Show Panel" };
                let response = draw_icon_button(ui, btn_icon, bg, fg, state.title_bar_state.toggle_panel_btn_hovered);
                state.title_bar_state.toggle_panel_btn_hovered = response.hovered();
                if response.clicked() { actions.push(TitleBarAction::TogglePanel); }
                if opacity > 0.8 { response.on_hover_text_at_pointer(btn_tooltip); }
                if !state.title_bar_state.header_visible {
                    let bg = BTN_NORMAL_BG.linear_multiply(opacity);
                    let fg = Color32::WHITE.linear_multiply(opacity);
                    let response = draw_icon_button(ui, &icons::SHOW_HEADER, bg, fg, false);
                    if response.clicked() { actions.push(TitleBarAction::ShowHeader); }
                    if opacity > 0.8 { response.on_hover_text_at_pointer(icons::SHOW_HEADER.tooltip); }
                }
            });
        });
    actions
}

// =============================================================================
// OUTER-BOX ROTATION
// =============================================================================

fn rotate_pos2_around(center: Pos2, p: Pos2, angle_rad: f32) -> Pos2 {
    let dx = p.x - center.x; let dy = p.y - center.y;
    let c = angle_rad.cos(); let s = angle_rad.sin();
    Pos2::new(center.x + dx * c - dy * s, center.y + dx * s + dy * c)
}

fn rect_aabb_after_rotate(center: Pos2, r: Rect, angle_rad: f32) -> Rect {
    let corners = [r.left_top(), r.right_top(), r.right_bottom(), r.left_bottom()];
    let rotated: [Pos2; 4] = [rotate_pos2_around(center, corners[0], angle_rad), rotate_pos2_around(center, corners[1], angle_rad), rotate_pos2_around(center, corners[2], angle_rad), rotate_pos2_around(center, corners[3], angle_rad)];
    let min_x = rotated.iter().map(|p| p.x).fold(f32::INFINITY, f32::min);
    let max_x = rotated.iter().map(|p| p.x).fold(f32::NEG_INFINITY, f32::max);
    let min_y = rotated.iter().map(|p| p.y).fold(f32::INFINITY, f32::min);
    let max_y = rotated.iter().map(|p| p.y).fold(f32::NEG_INFINITY, f32::max);
    Rect::from_min_max(Pos2::new(min_x, min_y), Pos2::new(max_x, max_y))
}

fn transform_shape_rotate_scale(shape: &mut Shape, center: Pos2, angle_rad: f32, scale: f32) {
    let no_rotate = angle_rad.abs() < 0.0001;
    let no_scale = (scale - 1.0).abs() < 0.0001;
    if no_rotate && no_scale { return; }
    let transform = |p: Pos2| -> Pos2 {
        let mut pt = p;
        if !no_rotate { pt = rotate_pos2_around(center, pt, angle_rad); }
        if !no_scale { pt = center + (pt - center) * scale; }
        pt
    };
    match shape {
        Shape::Vec(shapes) => { for s in shapes.iter_mut() { transform_shape_rotate_scale(s, center, angle_rad, scale); } }
        Shape::Circle(c) => { c.center = transform(c.center); c.radius *= scale; }
        Shape::Ellipse(e) => { e.center = transform(e.center); e.radius *= scale; }
        Shape::LineSegment { points, .. } => { points[0] = transform(points[0]); points[1] = transform(points[1]); }
        Shape::Path(p) => { for pt in p.points.iter_mut() { *pt = transform(*pt); } }
        Shape::Rect(r) => {
            r.rect = rect_aabb_after_rotate(center, r.rect, angle_rad);
            let min = center + (r.rect.min - center) * scale;
            let max = center + (r.rect.max - center) * scale;
            r.rect = Rect::from_min_max(min, max);
        }
        Shape::Text(t) => { t.pos = transform(t.pos); t.angle += angle_rad; }
        Shape::Mesh(mesh) => { for v in mesh.vertices.iter_mut() { v.pos = transform(v.pos); } }
        Shape::QuadraticBezier(b) => { for p in &mut b.points { *p = transform(*p); } }
        Shape::CubicBezier(b) => { for p in &mut b.points { *p = transform(*p); } }
        Shape::Callback(_) | Shape::Noop => {}
    }
}

fn transform_raw_input_for_rotation_scale(raw_input: &mut egui::RawInput, content_rect: Rect, angle_rad: f32, scale: f32) {
    let no_rotate = angle_rad.abs() < 0.0001;
    let no_scale = (scale - 1.0).abs() < 0.0001;
    if no_rotate && no_scale { return; }
    let center = content_rect.center();
    let inv_angle_rad = -angle_rad;
    let inv_scale = 1.0 / scale.max(0.1);
    for ev in raw_input.events.iter_mut() {
        let pos_opt: Option<&mut Pos2> = match ev {
            egui::Event::PointerMoved(pos) => Some(pos),
            egui::Event::PointerButton { pos, .. } => Some(pos),
            egui::Event::Touch { pos, .. } => Some(pos),
            _ => None,
        };
        if let Some(pos) = pos_opt {
            if content_rect.contains(*pos) {
                let mut p = *pos;
                if !no_scale { p = center + (p - center) * inv_scale; }
                if !no_rotate { p = rotate_pos2_around(center, p, inv_angle_rad); }
                *pos = p;
            }
        }
    }
}

fn transform_content_shapes(shapes: &[ClippedShape], content_rect: Rect, angle_rad: f32, scale: f32) -> Vec<ClippedShape> {
    if angle_rad.abs() < 0.0001 && (scale - 1.0).abs() < 0.0001 { return shapes.to_vec(); }
    let center = content_rect.center();
    let mut out = Vec::with_capacity(shapes.len());
    for clipped in shapes {
        let clip_center_y = clipped.clip_rect.center().y;
        if clip_center_y > TITLE_BAR_HEIGHT {
            let mut new_clip = clipped.clone();
            transform_shape_rotate_scale(&mut new_clip.shape, center, angle_rad, scale);
            new_clip.clip_rect = rect_aabb_after_rotate(center, new_clip.clip_rect, angle_rad);
            let min = center + (new_clip.clip_rect.min - center) * scale;
            let max = center + (new_clip.clip_rect.max - center) * scale;
            new_clip.clip_rect = Rect::from_min_max(min, max);
            new_clip.clip_rect = new_clip.clip_rect.expand(2.0);
            out.push(new_clip);
        } else {
            out.push(clipped.clone());
        }
    }
    out
}

// =============================================================================
// MAIN CONTENT RENDERER — with glowing card display
// =============================================================================

pub fn render_main_content(
    ctx: &Context,
    state: &mut AppState,
    shaper: &mut Option<(
        &mut cosmic_text::FontSystem,
        &mut cosmic_text::SwashCache,
        &mut HashMap<u64, egui::TextureHandle>,
    )>,
) {
    if state.title_bar_state.header_visible {
        egui::TopBottomPanel::bottom("footer_panel")
            .exact_height(24.0)
            .frame(egui::Frame::none().fill(Color32::from_black_alpha(20)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = egui::Vec2::new(12.0, 0.0);
                    ui.add_space(10.0);
                    if ui.small_button(RichText::new("◀").color(NEON_CYAN)).clicked() { state.prev_quote(); }
                    if ui.small_button(RichText::new("▶").color(NEON_CYAN)).clicked() { state.next_quote(); }
                    ui.separator();
                    ui.label(RichText::new("◈  NEURAL  FEED  ◈").font(FontId::proportional(8.5)).color(NEON_PLASMA.gamma_multiply(0.4)));
                    let visible_count = state.quotes.iter().filter(|q| !q.is_hidden).count();
                    let readout = format!("SYN:{:03}  •  VIS:{:03}  •  FREQ:{:04}ms  •  CORE:∞", state.quotes.len(), visible_count, state.rotation_interval.as_millis());
                    ui.label(RichText::new(readout).font(FontId::proportional(8.5)).color(NEON_SOLAR.gamma_multiply(0.4)));
                    ui.separator();
                    let dot_color = if state.rotation_enabled { Color32::from_rgb(80, 255, 120) } else { Color32::from_rgb(255, 60, 80) };
                    let (dot_rect, _) = ui.allocate_exact_size(Vec2::new(8.0, 8.0), Sense::hover());
                    ui.painter().circle_filled(dot_rect.center(), 3.0, dot_color);
                    ui.label(RichText::new(format!("Δt {}s  ·  {}", state.rotation_interval.as_secs(), if state.rotation_enabled { "STREAMING" } else { "PAUSED" })).color(Color32::from_rgba_unmultiplied(150, 200, 200, 180)).size(9.5));
                });
            });
    }

    if state.title_bar_state.control_panel_visible {
        egui::SidePanel::right("control_panel")
            .exact_width(CONTROL_PANEL_WIDTH)
            .resizable(false)
            .frame(Frame::none().fill(Color32::TRANSPARENT).inner_margin(egui::Margin { left: 10.0, right: 10.0, top: 15.0, bottom: 15.0 }))
            .show(ctx, |ui| { render_control_panel_contents(ui, state, shaper); });
    }

    egui::CentralPanel::default()
        .frame(Frame::none().fill(Color32::TRANSPARENT))
        .show(ctx, |ui| {
            // Background gradient/solid
            if !state.is_3d_bg_active {
                let draw_bg = state.theme.apply_to_entire_window || state.theme.mode == ThemeMode::Gradient;
                if draw_bg {
                    let rect = if state.theme.apply_to_entire_window { ctx.screen_rect() } else {
                        let mut r = ctx.screen_rect();
                        if state.title_bar_state.control_panel_visible { r.max.x -= CONTROL_PANEL_WIDTH; }
                        r
                    };
                    if state.theme.mode == ThemeMode::Solid {
                        ui.painter_at(rect).rect_filled(rect, Rounding::ZERO, state.theme.solid_color);
                    } else if !state.theme.gradient_colors.is_empty() {
                        let angle_rad = (state.theme.gradient_angle as f32).to_radians();
                        let dir = egui::Vec2::new(angle_rad.cos(), angle_rad.sin());
                        use egui::epaint::{Mesh, Vertex};
                        let mut mesh = Mesh::default();
                        let center = rect.center();
                        let project = |p: egui::Pos2| -> f32 { let v = p - center; v.x * dir.x + v.y * dir.y };
                        let p0 = project(rect.min); let p1 = project(egui::pos2(rect.max.x, rect.min.y));
                        let p2 = project(egui::pos2(rect.min.x, rect.max.y)); let p3 = project(rect.max);
                        let min_p = p0.min(p1).min(p2).min(p3);
                        let max_p = p0.max(p1).max(p2).max(p3);
                        let range = (max_p - min_p).max(0.1);
                        let calc_color = |p: f32| -> Color32 {
                            let t = ((p - min_p) / range).clamp(0.0, 1.0);
                            let colors = &state.theme.gradient_colors;
                            if colors.is_empty() { return Color32::TRANSPARENT; }
                            if colors.len() == 1 { return colors[0]; }
                            let n_segments = (colors.len() - 1) as f32;
                            let scaled_t = t * n_segments;
                            let mut index = scaled_t.floor() as usize;
                            index = index.min(colors.len() - 2);
                            let fract = scaled_t - index as f32;
                            let c1 = colors[index]; let c2 = colors[index + 1];
                            Color32::from_rgba_premultiplied(
                                (c1.r() as f32 * (1.0 - fract) + c2.r() as f32 * fract) as u8,
                                (c1.g() as f32 * (1.0 - fract) + c2.g() as f32 * fract) as u8,
                                (c1.b() as f32 * (1.0 - fract) + c2.b() as f32 * fract) as u8,
                                (c1.a() as f32 * (1.0 - fract) + c2.a() as f32 * fract) as u8,
                            )
                        };
                        let steps_x = 32; let steps_y = 32;
                        for yi in 0..=steps_y {
                            let ty = yi as f32 / steps_y as f32;
                            for xi in 0..=steps_x {
                                let tx = xi as f32 / steps_x as f32;
                                let p = rect.min + egui::vec2(rect.width() * tx, rect.height() * ty);
                                mesh.vertices.push(Vertex { pos: p, uv: egui::pos2(0.0, 0.0), color: calc_color(project(p)) });
                            }
                        }
                        for yi in 0..steps_y {
                            for xi in 0..steps_x {
                                let i0 = yi * (steps_x + 1) + xi; let i1 = i0 + 1;
                                let i2 = (yi + 1) * (steps_x + 1) + xi; let i3 = i2 + 1;
                                mesh.indices.extend_from_slice(&[i0, i1, i2, i1, i3, i2]);
                            }
                        }
                        ui.painter_at(rect).add(egui::Shape::mesh(mesh));
                    }
                }
            }

            egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(60.0);

                    let is_preview = !state.main_text_input.is_empty() || !state.sub_text_input.is_empty();

                    if is_preview {
                        // Preview mode: show the glowing card with input text
                        let preview_main = if !state.main_text_input.is_empty() {
                            state.main_text_input.clone()
                        } else {
                            "Type text to preview...".to_string()
                        };
                        let preview_sub = state.sub_text_input.clone();
                        render_quote_card(ctx, ui, &preview_main, &preview_sub, &state.text_style, state.title_bar_state.zoom_level, shaper, true);
                        ui.add_space(40.0);
                    } else if state.quotes.is_empty() {
                        ui.label(RichText::new("No quotes added yet!").color(Color32::GRAY).size(20.0));
                    } else {
                        // ── MAIN DISPLAY: show only current VISIBLE quote as a glowing card ──
                        // Find the current visible quote to show
                        let display_quote = if !state.quotes[state.current_quote_index].is_hidden {
                            Some(state.current_quote_index)
                        } else {
                            // Current is hidden, find next visible
                            state.next_visible_index(state.current_quote_index)
                        };

                        if let Some(idx) = display_quote {
                            let main_text = state.quotes[idx].main_text.clone();
                            let sub_text = state.quotes[idx].sub_text.clone();
                            render_quote_card(ctx, ui, &main_text, &sub_text, &state.text_style, state.title_bar_state.zoom_level, shaper, false);
                        } else {
                            // All hidden
                            ui.add_space(40.0);
                            let msg_rect = egui::Rect::from_center_size(
                                ui.cursor().center() + Vec2::new(0.0, 20.0),
                                Vec2::new(320.0, 60.0),
                            );
                            ui.painter().rect_filled(msg_rect, Rounding::same(12.0), Color32::from_rgba_unmultiplied(30, 30, 50, 180));
                            ui.painter().rect_stroke(msg_rect, Rounding::same(12.0), Stroke::new(1.0, NEON_SOLAR.gamma_multiply(0.5)));
                            ui.painter().text(msg_rect.center(), egui::Align2::CENTER_CENTER, "All quotes are hidden", FontId::proportional(16.0), NEON_SOLAR.gamma_multiply(0.8));
                        }

                        ui.add_space(40.0);
                    }
                });
            });
        });
}

/// Render a single quote as a beautiful glowing semi-rounded card
fn render_quote_card(
    ctx: &Context,
    ui: &mut egui::Ui,
    main_text: &str,
    sub_text: &str,
    text_style: &TextStyleConfig,
    zoom_level: f32,
    shaper: &mut Option<(
        &mut cosmic_text::FontSystem,
        &mut cosmic_text::SwashCache,
        &mut HashMap<u64, egui::TextureHandle>,
    )>,
    is_preview: bool,
) {
    let card_width = (ui.available_width() * 0.88).min(700.0).max(300.0);
    let main_size = text_style.main_text_size * zoom_level;
    let sub_size = text_style.sub_text_size * zoom_level;

    // Card outer glow (multi-layered)
    let card_pos = ui.cursor().min;
    let glow_color_outer = NEON_CYAN.gamma_multiply(0.06);
    let glow_color_inner = NEON_CYAN.gamma_multiply(0.12);
    let border_color = NEON_CYAN.gamma_multiply(0.35);
    let card_bg = Color32::from_rgba_unmultiplied(8, 16, 32, 200);

    // We use a vertical group to measure and then draw the card behind
    let resp = ui.allocate_ui_with_layout(
        Vec2::new(card_width, 0.0),
        egui::Layout::top_down(egui::Align::Center),
        |ui| {
            ui.set_width(card_width);

            // Outer glow layers
            let pad = 28.0;
            let inner_pad = 32.0;

            // Spacer top
            ui.add_space(inner_pad);

            // Main text
            let main_color = if is_preview && main_text == "Type text to preview..." {
                Color32::WHITE.linear_multiply(0.4)
            } else {
                text_style.main_text_color
            };

            let mut rendered_shaped_main = false;
            if contains_bengali(main_text) {
                if let Some((ref mut fs, ref mut sc, ref mut tc)) = shaper {
                    if let Some((tex_id, size)) = render_shaped_text(ctx, fs, sc, main_text, main_size, main_color, tc) {
                        ui.add(egui::Image::new(egui::load::SizedTexture::new(tex_id, size)).sense(egui::Sense::hover()));
                        rendered_shaped_main = true;
                    }
                }
            }
            if !rendered_shaped_main {
                ui.add(egui::Label::new(
                    RichText::new(main_text).color(main_color).size(main_size).strong()
                ).sense(egui::Sense::hover()));
            }

            ui.add_space(text_style.between_gap);

            // Sub text separator line
            if !sub_text.is_empty() {
                let sep_rect = ui.cursor();
                let sep_w = card_width * 0.3;
                let sep_x = sep_rect.center().x;
                ui.painter().line_segment(
                    [egui::pos2(sep_x - sep_w / 2.0, sep_rect.top()), egui::pos2(sep_x + sep_w / 2.0, sep_rect.top())],
                    Stroke::new(1.0, NEON_CYAN.gamma_multiply(0.3)),
                );
                ui.add_space(6.0);

                let sub_color = text_style.sub_text_color;
                let mut rendered_shaped_sub = false;
                if contains_bengali(sub_text) {
                    if let Some((ref mut fs, ref mut sc, ref mut tc)) = shaper {
                        if let Some((tex_id, size)) = render_shaped_text(ctx, fs, sc, sub_text, sub_size, sub_color, tc) {
                            ui.add(egui::Image::new(egui::load::SizedTexture::new(tex_id, size)).sense(egui::Sense::hover()));
                            rendered_shaped_sub = true;
                        }
                    }
                }
                if !rendered_shaped_sub {
                    ui.add(egui::Label::new(
                        RichText::new(sub_text).color(sub_color).size(sub_size)
                    ).sense(egui::Sense::hover()));
                }
            }

            ui.add_space(inner_pad);
        },
    );

    // Now draw the card background + borders behind the text
    let card_rect = resp.response.rect;

    // Multi-layer glow effect
    for (expand, alpha) in [(18.0, 0.03_f32), (10.0, 0.06), (4.0, 0.10)] {
        let glow = card_rect.expand(expand);
        ui.painter().rect_filled(glow, Rounding::same(20.0 + expand), NEON_CYAN.gamma_multiply(alpha));
    }

    // Card background
    ui.painter().rect_filled(card_rect, Rounding::same(18.0), card_bg);

    // Inner rim highlight (top edge glass effect)
    ui.painter().line_segment(
        [egui::pos2(card_rect.left() + 20.0, card_rect.top() + 1.5), egui::pos2(card_rect.right() - 20.0, card_rect.top() + 1.5)],
        Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 30)),
    );

    // Outer border
    ui.painter().rect_stroke(card_rect, Rounding::same(18.0), Stroke::new(1.5, border_color));

    // Corner accent marks
    let c_len = 12.0;
    let c_stroke = Stroke::new(2.0, NEON_CYAN.gamma_multiply(0.7));
    let tl = card_rect.left_top();
    let tr = card_rect.right_top();
    let bl = card_rect.left_bottom();
    let br = card_rect.right_bottom();
    ui.painter().line_segment([egui::pos2(tl.x + 8.0, tl.y), egui::pos2(tl.x + 8.0 + c_len, tl.y)], c_stroke);
    ui.painter().line_segment([egui::pos2(tl.x, tl.y + 8.0), egui::pos2(tl.x, tl.y + 8.0 + c_len)], c_stroke);
    ui.painter().line_segment([egui::pos2(tr.x - 8.0, tr.y), egui::pos2(tr.x - 8.0 - c_len, tr.y)], c_stroke);
    ui.painter().line_segment([egui::pos2(tr.x, tr.y + 8.0), egui::pos2(tr.x, tr.y + 8.0 + c_len)], c_stroke);
    ui.painter().line_segment([egui::pos2(bl.x + 8.0, bl.y), egui::pos2(bl.x + 8.0 + c_len, bl.y)], c_stroke);
    ui.painter().line_segment([egui::pos2(bl.x, bl.y - 8.0), egui::pos2(bl.x, bl.y - 8.0 - c_len)], c_stroke);
    ui.painter().line_segment([egui::pos2(br.x - 8.0, br.y), egui::pos2(br.x - 8.0 - c_len, br.y)], c_stroke);
    ui.painter().line_segment([egui::pos2(br.x, br.y - 8.0), egui::pos2(br.x, br.y - 8.0 - c_len)], c_stroke);
}

// =============================================================================
// CONTROL PANEL RENDERER
// =============================================================================

pub fn render_control_panel_contents(
    ui: &mut egui::Ui,
    state: &mut AppState,
    shaper: &mut Option<(
        &mut cosmic_text::FontSystem,
        &mut cosmic_text::SwashCache,
        &mut HashMap<u64, egui::TextureHandle>,
    )>,
) {
    ui.set_max_width(ui.available_width());
    let panel_content_width = CONTROL_PANEL_WIDTH - 20.0;
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .enable_scrolling(true)
        .show(ui, |ui| {
            ui.set_width(panel_content_width);
            ui.set_max_width(panel_content_width);

            // Panel Font Color
            ui.horizontal(|ui| {
                let color = state.text_style.panel_text_color;
                label_with_glow(ui, "Panel Font Color:", color, 10.5, color.gamma_multiply(0.25), egui::Align2::LEFT_CENTER);
                ui.add_space(4.0);
                ui.scope(|ui| {
                    ui.spacing_mut().interact_size = egui::Vec2::new(14.0, 14.0);
                    let mut color_arr = [state.text_style.panel_text_color.r(), state.text_style.panel_text_color.g(), state.text_style.panel_text_color.b(), 255u8];
                    if ui.color_edit_button_srgba_unmultiplied(&mut color_arr).changed() {
                        state.text_style.panel_text_color = Color32::from_rgb(color_arr[0], color_arr[1], color_arr[2]);
                        state.save();
                    }
                });
            });
            ui.add_space(10.0);

            // Add Custom Text Section
            render_section(ui, &format!("ADD CUSTOM TEXT  [{}]", state.quotes.len() + 1), state.text_style.panel_text_color, |ui| {
                ui.horizontal(|ui| {
                    let text_width = (panel_content_width - 80.0).max(50.0);
                    let mut text_response = None;
                    egui::Frame::none()
                        .fill(Color32::TRANSPARENT)
                        .stroke(Stroke::new(1.0, NEON_CYAN.gamma_multiply(0.2)))
                        .rounding(Rounding::same(4.0))
                        .show(ui, |ui| {
                            let resp = ui.add(egui::TextEdit::multiline(&mut state.main_text_input).hint_text("Main text... (Enter to submit, Shift+Enter for new line)").desired_rows(3).desired_width(text_width).lock_focus(true).frame(false));
                            text_response = Some(resp);
                        });
                    let text_response = text_response.unwrap();
                    if text_response.changed() { ui.ctx().request_repaint(); }
                    if text_response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter) && !i.modifiers.shift) {
                        if !state.main_text_input.trim().is_empty() {
                            state.add_quote(state.main_text_input.clone(), state.sub_text_input.clone());
                            state.save();
                            state.main_text_input.clear();
                            state.sub_text_input.clear();
                            text_response.request_focus();
                        }
                    }
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            if ui.small_button(RichText::new("A+").color(state.text_style.panel_text_color).size(10.5)).clicked() && state.text_style.main_text_size < 100.0 { state.text_style.main_text_size += 2.0; state.save(); }
                            let color_btn = ui.add(egui::Button::new(RichText::new("🎨").color(state.text_style.panel_text_color).size(13.0)).fill(Color32::from_rgb(244, 67, 54)).stroke(Stroke::new(1.0, Color32::WHITE.gamma_multiply(0.4))).min_size(Vec2::new(24.0, 20.0)));
                            if color_btn.clicked() { state.show_main_color_picker = !state.show_main_color_picker; }
                        });
                        if ui.small_button(RichText::new("A-").color(state.text_style.panel_text_color).size(10.5)).clicked() && state.text_style.main_text_size > 12.0 { state.text_style.main_text_size -= 2.0; state.save(); }
                    });
                });
                if state.show_main_color_picker {
                    egui::Frame::none().fill(Color32::from_black_alpha(40)).stroke(Stroke::new(1.0, NEON_CYAN.gamma_multiply(0.25))).inner_margin(Vec2::new(8.0, 8.0)).rounding(Rounding::same(4.0)).show(ui, |ui| {
                        let mut color_arr = [state.text_style.main_text_color.r(), state.text_style.main_text_color.g(), state.text_style.main_text_color.b(), 255u8];
                        if ui.color_edit_button_srgba_unmultiplied(&mut color_arr).changed() { state.text_style.main_text_color = Color32::from_rgb(color_arr[0], color_arr[1], color_arr[2]); state.save(); }
                    });
                }
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    let text_width = (panel_content_width - 80.0).max(50.0);
                    let mut sub_response = None;
                    egui::Frame::none().fill(Color32::TRANSPARENT).stroke(Stroke::new(1.0, NEON_CYAN.gamma_multiply(0.2))).rounding(Rounding::same(4.0)).show(ui, |ui| {
                        let resp = ui.add(egui::TextEdit::multiline(&mut state.sub_text_input).hint_text("Supporting text... (Enter to submit, Shift+Enter for new line)").desired_rows(2).desired_width(text_width).frame(false));
                        sub_response = Some(resp);
                    });
                    let sub_response = sub_response.unwrap();
                    if sub_response.changed() { ui.ctx().request_repaint(); }
                    if sub_response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter) && !i.modifiers.shift) {
                        if !state.main_text_input.trim().is_empty() {
                            state.add_quote(state.main_text_input.clone(), state.sub_text_input.clone());
                            state.save(); state.main_text_input.clear(); state.sub_text_input.clear();
                        }
                    }
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            if ui.small_button(RichText::new("A+").color(state.text_style.panel_text_color).size(10.5)).clicked() && state.text_style.sub_text_size < 50.0 { state.text_style.sub_text_size += 1.0; state.save(); }
                            let color_btn = ui.add(egui::Button::new(RichText::new("🎨").color(state.text_style.panel_text_color).size(13.0)).fill(Color32::from_rgb(244, 67, 54)).stroke(Stroke::new(1.0, Color32::WHITE.gamma_multiply(0.4))).min_size(Vec2::new(24.0, 20.0)));
                            if color_btn.clicked() { state.show_sub_color_picker = !state.show_sub_color_picker; }
                        });
                        if ui.small_button(RichText::new("A-").color(state.text_style.panel_text_color).size(10.5)).clicked() && state.text_style.sub_text_size > 8.0 { state.text_style.sub_text_size -= 1.0; state.save(); }
                    });
                });
                if state.show_sub_color_picker {
                    egui::Frame::none().fill(Color32::from_black_alpha(40)).stroke(Stroke::new(1.0, NEON_CYAN.gamma_multiply(0.25))).inner_margin(Vec2::new(8.0, 8.0)).rounding(Rounding::same(4.0)).show(ui, |ui| {
                        let mut color_arr = [state.text_style.sub_text_color.r(), state.text_style.sub_text_color.g(), state.text_style.sub_text_color.b(), 255u8];
                        if ui.color_edit_button_srgba_unmultiplied(&mut color_arr).changed() { state.text_style.sub_text_color = Color32::from_rgb(color_arr[0], color_arr[1], color_arr[2]); state.save(); }
                    });
                }
                ui.add_space(8.0);
                let add_btn_color = Color32::from_rgb(76, 175, 80);
                if draw_text_button(ui, "+ Add Text", add_btn_color, ui.available_width() - 8.0, 32.0).clicked() {
                    if !state.main_text_input.is_empty() {
                        state.add_quote(state.main_text_input.clone(), state.sub_text_input.clone());
                        state.save(); state.main_text_input.clear(); state.sub_text_input.clear();
                    }
                }
            });

            ui.add_space(10.0);

            // Line Gaps Section
            render_section(ui, "LINE GAPS", state.text_style.panel_text_color, |ui| {
                ui.horizontal(|ui| {
                    label_with_glow(ui, "Main Text Gap", state.text_style.panel_text_color, 10.5, state.text_style.panel_text_color.gamma_multiply(0.25), egui::Align2::LEFT_CENTER);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        label_with_glow(ui, &format!("{:.1}", state.text_style.main_line_gap), NEON_LIME, 10.5, Color32::from_black_alpha(120), egui::Align2::RIGHT_CENTER);
                        let slider_width = ui.available_width();
                        if ui.add_sized([slider_width, ui.available_height()], egui::Slider::new(&mut state.text_style.main_line_gap, 1.0..=3.0).step_by(0.1).text("")).changed() { state.save(); }
                    });
                });
                ui.horizontal(|ui| {
                    label_with_glow(ui, "Supporting Text Gap", state.text_style.panel_text_color, 10.5, state.text_style.panel_text_color.gamma_multiply(0.25), egui::Align2::LEFT_CENTER);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        label_with_glow(ui, &format!("{:.1}", state.text_style.sub_line_gap), NEON_LIME, 10.5, Color32::from_black_alpha(120), egui::Align2::RIGHT_CENTER);
                        let slider_width = ui.available_width();
                        if ui.add_sized([slider_width, ui.available_height()], egui::Slider::new(&mut state.text_style.sub_line_gap, 1.0..=3.0).step_by(0.1).text("")).changed() { state.save(); }
                    });
                });
                ui.horizontal(|ui| {
                    label_with_glow(ui, "Gap Between Texts", state.text_style.panel_text_color, 10.5, state.text_style.panel_text_color.gamma_multiply(0.25), egui::Align2::LEFT_CENTER);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        label_with_glow(ui, &format!("{:.0} px", state.text_style.between_gap), NEON_LIME, 10.5, Color32::from_black_alpha(120), egui::Align2::RIGHT_CENTER);
                        let slider_width = ui.available_width();
                        if ui.add_sized([slider_width, ui.available_height()], egui::Slider::new(&mut state.text_style.between_gap, 0.0..=50.0).step_by(1.0).text("")).changed() { state.save(); }
                    });
                });
            });

            ui.add_space(10.0);

            // Interval Section
            render_section(ui, "INTERVAL (SECONDS)", state.text_style.panel_text_color, |ui| {
                ui.horizontal(|ui| {
                    let frame_response = egui::Frame::none().fill(Color32::TRANSPARENT).stroke(Stroke::new(1.0, NEON_CYAN.gamma_multiply(0.4))).rounding(Rounding::same(4.0)).show(ui, |ui| ui.add(egui::DragValue::new(&mut state.interval_secs).range(1..=60)));
                    let interval_resp = frame_response.inner;
                    if interval_resp.changed() { state.interval_secs = state.interval_secs.clamp(1, 60); }
                    if interval_resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) { state.rotation_interval = Duration::from_secs(state.interval_secs); state.last_rotation = Instant::now(); state.save(); }
                    label_with_glow(ui, "seconds", state.text_style.panel_text_color, 10.5, state.text_style.panel_text_color.gamma_multiply(0.25), egui::Align2::LEFT_CENTER);
                });
                ui.add_space(8.0);
                if draw_text_button(ui, "Set Interval", Color32::from_rgb(33, 150, 243), ui.available_width() - 8.0, 28.0).clicked() {
                    let clamped = state.interval_secs.clamp(1, 60);
                    state.interval_secs = clamped; state.rotation_interval = Duration::from_secs(clamped);
                    state.last_rotation = Instant::now(); state.save(); ui.ctx().request_repaint();
                }
                ui.add_space(8.0);
                let (toggle_text, toggle_color) = if state.rotation_enabled { ("⏸ Pause Rotation", Color32::from_rgb(255, 152, 0)) } else { ("▶ Resume Rotation", Color32::from_rgb(76, 175, 80)) };
                if draw_text_button(ui, toggle_text, toggle_color, ui.available_width() - 8.0, 28.0).clicked() {
                    state.rotation_enabled = !state.rotation_enabled;
                    if state.rotation_enabled { state.last_rotation = Instant::now(); }
                }
            });

            ui.add_space(10.0);

            // ===== TEXT LIST with drag-to-reorder + compact hide/delete =====
            render_section(ui, &format!("TEXT LIST ({})", state.quotes.len()), state.text_style.panel_text_color, |ui| {
                let mut to_delete: Option<usize> = None;
                let mut to_select: Option<usize> = None;
                let mut to_toggle_hide: Option<usize> = None;
                let mut move_from_to: Option<(usize, usize)> = None;

                let list_box_internal_width = (panel_content_width - 48.0).max(10.0);
                let n = state.quotes.len();

                for idx in 0..n {
                    let is_current = idx == state.current_quote_index;
                    let is_hidden = state.quotes[idx].is_hidden;
                    let bg_color = if is_current { Color32::from_black_alpha(45) } else { Color32::from_black_alpha(20) };

                    // Drag handle + up/down area — left side
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 2.0;

                        // ── UP / DOWN reorder buttons (compact vertical stack) ──
                        ui.vertical(|ui| {
                            ui.spacing_mut().item_spacing.y = 1.0;
                            let up_resp = ui.add_sized(
                                Vec2::new(14.0, 13.0),
                                egui::Button::new(RichText::new("▲").size(8.0).color(NEON_CYAN.gamma_multiply(if idx > 0 { 0.9 } else { 0.3 }))).fill(Color32::TRANSPARENT).frame(false),
                            );
                            if up_resp.clicked() && idx > 0 { move_from_to = Some((idx, idx - 1)); }
                            up_resp.on_hover_text("Move up");

                            let dn_resp = ui.add_sized(
                                Vec2::new(14.0, 13.0),
                                egui::Button::new(RichText::new("▼").size(8.0).color(NEON_CYAN.gamma_multiply(if idx + 1 < n { 0.9 } else { 0.3 }))).fill(Color32::TRANSPARENT).frame(false),
                            );
                            if dn_resp.clicked() && idx + 1 < n { move_from_to = Some((idx, idx + 1)); }
                            dn_resp.on_hover_text("Move down");
                        });

                        // ── Main list item box ──
                        egui::Frame::none()
                            .fill(bg_color)
                            .inner_margin(egui::Margin { left: 6.0, right: 4.0, top: 5.0, bottom: 5.0 })
                            .rounding(Rounding::same(5.0))
                            .stroke(Stroke::new(
                                if is_current { 1.5 } else { 1.0 },
                                if is_current { NEON_CYAN.gamma_multiply(0.5) } else { NEON_CYAN.gamma_multiply(0.18) },
                            ))
                            .show(ui, |ui| {
                                let box_w = (list_box_internal_width - 20.0).max(10.0); // subtract arrow column
                                ui.set_max_width(box_w);
                                ui.set_width(box_w);

                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    // ── COMPACT action buttons: hide + delete, zero gap ──
                                    ui.spacing_mut().item_spacing.x = 1.0; // ~75% less gap

                                    // Delete button (🗑 icon, compact)
                                    let del_btn = ui.add(
                                        egui::Button::new(RichText::new("✕").color(NEON_ROSE.gamma_multiply(0.85)).size(11.0))
                                            .fill(Color32::TRANSPARENT)
                                            .min_size(Vec2::new(18.0, 18.0))
                                            .frame(false),
                                    ).on_hover_text("Delete");
                                    if del_btn.clicked() { to_delete = Some(idx); }

                                    // Hide/Unhide button (eye icon, compact)
                                    let (hide_sym, hide_tip, hide_col) = if is_hidden {
                                        ("👁", "Show in display", NEON_SOLAR.gamma_multiply(0.9))
                                    } else {
                                        ("◉", "Hide from display", Color32::from_rgba_unmultiplied(180, 200, 220, 180))
                                    };
                                    let hide_btn = ui.add(
                                        egui::Button::new(RichText::new(hide_sym).color(hide_col).size(11.0))
                                            .fill(Color32::TRANSPARENT)
                                            .min_size(Vec2::new(18.0, 18.0))
                                            .frame(false),
                                    ).on_hover_text(hide_tip);
                                    if hide_btn.clicked() { to_toggle_hide = Some(idx); }

                                    // Text area
                                    ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
                                        ui.vertical(|ui| {
                                            let text_color = state.text_style.panel_text_color;
                                            let alpha_mult = if is_hidden { 0.4 } else { 1.0 };
                                            let display_color = text_color.linear_multiply(alpha_mult);
                                            let main_text_size = 13.0;
                                            let sub_text_size = 11.5;

                                            // Hidden badge
                                            if is_hidden {
                                                ui.label(RichText::new("⊘ HIDDEN").size(8.5).color(NEON_SOLAR.gamma_multiply(0.7)));
                                            }

                                            let display_main = format!("{}. {}", idx + 1, &state.quotes[idx].main_text);
                                            let mut clicked_main = false;
                                            if contains_bengali(&state.quotes[idx].main_text) && shaper.is_some() {
                                                if let Some((ref mut fs, ref mut sc, ref mut tc)) = shaper {
                                                    if let Some((tex_id, size)) = render_shaped_text(ui.ctx(), fs, sc, &display_main, main_text_size, display_color, tc) {
                                                        let available_w = ui.available_width();
                                                        let mut show_ellipsis = false;
                                                        let mut display_size = size;
                                                        let mut uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
                                                        if size.x > available_w { show_ellipsis = true; let ellipsis_w = 12.0; display_size.x = (available_w - ellipsis_w).max(1.0); uv.max.x = display_size.x / size.x; }
                                                        clicked_main = ui.horizontal(|ui| {
                                                            ui.spacing_mut().item_spacing.x = 0.0;
                                                            let resp = ui.add(egui::Image::new(egui::load::SizedTexture::new(tex_id, display_size)).uv(uv).sense(egui::Sense::click()));
                                                            if show_ellipsis { ui.add(egui::Label::new(RichText::new("...").color(display_color).size(main_text_size))); }
                                                            resp.clicked()
                                                        }).inner;
                                                    } else {
                                                        clicked_main = ui.add(egui::Label::new(RichText::new(&display_main).color(display_color).size(main_text_size)).truncate()).clicked();
                                                    }
                                                }
                                            } else {
                                                clicked_main = ui.add(egui::Label::new(RichText::new(&display_main).color(display_color).size(main_text_size)).truncate()).clicked();
                                            }

                                            let display_sub = format!("↳ {}", &state.quotes[idx].sub_text);
                                            let sub_color = display_color.gamma_multiply(0.65);
                                            if contains_bengali(&state.quotes[idx].sub_text) && shaper.is_some() {
                                                if let Some((ref mut fs, ref mut sc, ref mut tc)) = shaper {
                                                    if let Some((tex_id, size)) = render_shaped_text(ui.ctx(), fs, sc, &display_sub, sub_text_size, sub_color, tc) {
                                                        let available_w = ui.available_width();
                                                        let mut show_ellipsis = false; let mut display_size = size;
                                                        let mut uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
                                                        if size.x > available_w { show_ellipsis = true; let ellipsis_w = 12.0; display_size.x = (available_w - ellipsis_w).max(1.0); uv.max.x = display_size.x / size.x; }
                                                        ui.horizontal(|ui| { ui.spacing_mut().item_spacing.x = 0.0; ui.add(egui::Image::new(egui::load::SizedTexture::new(tex_id, display_size)).uv(uv)); if show_ellipsis { ui.add(egui::Label::new(RichText::new("...").color(sub_color).size(sub_text_size))); } });
                                                    } else {
                                                        ui.add(egui::Label::new(RichText::new(&display_sub).color(sub_color).size(sub_text_size)).truncate());
                                                    }
                                                }
                                            } else {
                                                ui.add(egui::Label::new(RichText::new(&display_sub).color(sub_color).size(sub_text_size)).truncate());
                                            }

                                            if clicked_main { to_select = Some(idx); }
                                        });
                                    });
                                });
                            });
                    });

                    ui.add_space(3.0);
                }

                // Apply deferred mutations
                if let Some((from, to)) = move_from_to { state.move_quote(from, to); }
                if let Some(idx) = to_delete { state.delete_quote(idx); state.save(); }
                if let Some(idx) = to_select { state.current_quote_index = idx; state.last_rotation = Instant::now(); }
                if let Some(idx) = to_toggle_hide {
                    state.quotes[idx].is_hidden = !state.quotes[idx].is_hidden;
                    // If we just hid the current quote, advance to next visible
                    if state.quotes[idx].is_hidden && state.current_quote_index == idx {
                        if let Some(next) = state.next_visible_index(idx) {
                            state.current_quote_index = next;
                        }
                    }
                    state.save();
                }
            });

            ui.add_space(10.0);

            // Clear All
            if !state.confirm_clear_pending {
                if draw_text_button(ui, "Clear All", Color32::from_rgb(255, 152, 0), ui.available_width(), 28.0).clicked() { state.confirm_clear_pending = true; }
            } else {
                ui.horizontal(|ui| {
                    label_with_glow(ui, "Are you sure?", state.text_style.panel_text_color, 11.0, state.text_style.panel_text_color.gamma_multiply(0.25), egui::Align2::LEFT_CENTER);
                    if ui.button(RichText::new("Yes, Clear").color(Color32::WHITE).size(10.5)).clicked() { state.quotes.clear(); state.current_quote_index = 0; state.confirm_clear_pending = false; state.save(); }
                    if ui.button(RichText::new("Cancel").color(Color32::from_rgba_unmultiplied(190, 190, 215, 255)).size(10.5)).clicked() { state.confirm_clear_pending = false; }
                });
            }

            ui.add_space(10.0);

            // Info Section
            egui::Frame::none()
                .fill(Color32::from_black_alpha(26))
                .stroke(egui::Stroke::new(1.0, NEON_CYAN.gamma_multiply(0.22)))
                .inner_margin(Vec2::new(10.0, 10.0))
                .rounding(Rounding::same(4.0))
                .show(ui, |ui| {
                    let info_color = state.text_style.panel_text_color;
                    let shadow = info_color.gamma_multiply(0.25);
                    let visible_count = state.quotes.iter().filter(|q| !q.is_hidden).count();
                    label_with_glow(ui, &format!("Current Interval: {}s", state.rotation_interval.as_secs()), info_color, 10.5, shadow, egui::Align2::LEFT_CENTER);
                    label_with_glow(ui, &format!("Total: {}  •  Visible: {}  •  Hidden: {}", state.quotes.len(), visible_count, state.quotes.len() - visible_count), info_color, 10.5, shadow, egui::Align2::LEFT_CENTER);
                    label_with_glow(ui, &format!("Rotation: {}", if state.rotation_enabled { "Active" } else { "Paused" }), info_color, 10.5, shadow, egui::Align2::LEFT_CENTER);
                });
        });
}

fn render_section(ui: &mut egui::Ui, title: &str, text_color: Color32, add_contents: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::none()
        .fill(Color32::TRANSPARENT)
        .stroke(Stroke::new(1.0, NEON_CYAN.gamma_multiply(0.25)))
        .inner_margin(egui::Margin::same(1.0))
        .rounding(Rounding::same(10.0))
        .show(ui, |ui| {
            egui::Frame::none()
                .fill(Color32::TRANSPARENT)
                .stroke(Stroke::new(0.5, Color32::from_white_alpha(12)))
                .inner_margin(egui::Margin { left: 12.0, right: 12.0, top: 10.0, bottom: 12.0 })
                .rounding(Rounding::same(9.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let (mark_rect, _) = ui.allocate_exact_size(Vec2::new(3.0, 12.0), Sense::hover());
                        ui.painter().rect_filled(mark_rect, Rounding::same(2.0), NEON_LIME);
                        ui.add_space(2.0);
                        label_with_glow(ui, title, text_color, 10.0, text_color.gamma_multiply(0.4), egui::Align2::LEFT_CENTER);
                        let avail = ui.available_width();
                        if avail > 4.0 {
                            let (line_rect, _) = ui.allocate_exact_size(Vec2::new(avail - 2.0, 1.0), Sense::hover());
                            let mid_y = line_rect.center().y;
                            ui.painter().line_segment([egui::pos2(line_rect.left(), mid_y), egui::pos2(line_rect.right(), mid_y)], Stroke::new(0.5, NEON_LIME.gamma_multiply(0.17)));
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

pub fn render_theme_modal(ctx: &Context, state: &mut AppState) {
    if !state.theme_modal_open { return; }
    egui::Window::new("Customize Theme")
        .collapsible(false).resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, Vec2::new(0.0, 0.0))
        .fixed_size(Vec2::new(400.0, 500.0))
        .frame(egui::Frame::window(&ctx.style()).fill(Color32::from_white_alpha(15)))
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Mode:").color(Color32::WHITE).size(12.0));
                let gradient_selected = state.theme.mode == ThemeMode::Gradient;
                let solid_selected = state.theme.mode == ThemeMode::Solid;
                if ui.selectable_label(gradient_selected, "Gradient").clicked() { state.theme.mode = ThemeMode::Gradient; state.save(); }
                if ui.selectable_label(solid_selected, "Solid").clicked() { state.theme.mode = ThemeMode::Solid; state.save(); }
            });
            ui.add_space(10.0);
            ui.horizontal(|ui| { if ui.checkbox(&mut state.theme.apply_to_entire_window, "Apply to Entire Window").changed() { state.save(); } });
            ui.add_space(15.0);
            if state.theme.mode == ThemeMode::Gradient {
                ui.label(RichText::new("Gradient Angle:").color(Color32::WHITE).size(12.0));
                ui.add_space(5.0);
                ui.horizontal_wrapped(|ui| {
                    for angle in [0, 45, 90, 135, 180, 225, 270, 315] {
                        let selected = state.theme.gradient_angle == angle;
                        if ui.selectable_label(selected, format!("{}°", angle)).clicked() { state.theme.gradient_angle = angle; state.save(); }
                    }
                });
                ui.add_space(15.0);
                ui.label(RichText::new("Gradient Colors:").color(Color32::WHITE).size(12.0));
                ui.add_space(5.0);
                let mut to_remove = None;
                for idx in 0..state.theme.gradient_colors.len() {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(format!("Color {}:", idx + 1)).color(Color32::GRAY).size(11.0));
                        let color = state.theme.gradient_colors[idx];
                        let mut color_array = [color.r() as f32 / 255.0, color.g() as f32 / 255.0, color.b() as f32 / 255.0, 1.0];
                        if ui.color_edit_button_rgba_unmultiplied(&mut color_array).changed() {
                            state.theme.gradient_colors[idx] = Color32::from_rgb((color_array[0] * 255.0) as u8, (color_array[1] * 255.0) as u8, (color_array[2] * 255.0) as u8);
                            state.save();
                        }
                        if state.theme.gradient_colors.len() > 2 {
                            if ui.add(egui::Button::new(RichText::new("Remove").color(Color32::WHITE).size(10.0)).fill(Color32::from_rgb(255, 70, 70))).clicked() { to_remove = Some(idx); }
                        }
                    });
                }
                if let Some(idx) = to_remove { state.theme.gradient_colors.remove(idx); state.save(); }
                if state.theme.gradient_colors.len() < 5 { if ui.button("+ Add Color").clicked() { state.theme.gradient_colors.push(Color32::WHITE); state.save(); } }
                ui.add_space(15.0);
                ui.label(RichText::new("Preset Gradients:").color(Color32::WHITE).size(12.0));
                ui.add_space(5.0);
                ui.horizontal_wrapped(|ui| {
                    if ui.button("⬡ Aurora Void").clicked() { state.theme.gradient_colors = vec![Color32::from_rgb(2, 4, 16), Color32::from_rgb(30, 0, 80), Color32::from_rgb(0, 60, 120), Color32::from_rgb(0, 200, 180)]; state.save(); }
                    if ui.button("⬡ Solar Flare").clicked() { state.theme.gradient_colors = vec![Color32::from_rgb(10, 0, 30), Color32::from_rgb(120, 20, 0), Color32::from_rgb(255, 100, 0), Color32::from_rgb(255, 220, 60)]; state.save(); }
                });
                ui.horizontal_wrapped(|ui| {
                    if ui.button("⬡ Plasma Storm").clicked() { state.theme.gradient_colors = vec![Color32::from_rgb(5, 0, 20), Color32::from_rgb(80, 0, 180), Color32::from_rgb(200, 0, 255), Color32::from_rgb(255, 80, 200)]; state.save(); }
                    if ui.button("⬡ Deep Ocean").clicked() { state.theme.gradient_colors = vec![Color32::from_rgb(0, 5, 20), Color32::from_rgb(0, 30, 80), Color32::from_rgb(0, 100, 160), Color32::from_rgb(0, 200, 220)]; state.save(); }
                });
                ui.horizontal_wrapped(|ui| {
                    if ui.button("⬡ Matrix Rain").clicked() { state.theme.gradient_colors = vec![Color32::from_rgb(0, 8, 0), Color32::from_rgb(0, 40, 10), Color32::from_rgb(0, 120, 30), Color32::from_rgb(80, 255, 100)]; state.save(); }
                    if ui.button("⬡ Quantum Noir").clicked() { state.theme.gradient_colors = vec![Color32::from_rgb(2, 2, 6), Color32::from_rgb(10, 10, 25), Color32::from_rgb(25, 25, 50), Color32::from_rgb(60, 60, 100)]; state.save(); }
                });
            } else {
                ui.label(RichText::new("Solid Color:").color(Color32::WHITE).size(12.0));
                ui.add_space(5.0);
                let solid = state.theme.solid_color;
                let mut color_array = [solid.r() as f32 / 255.0, solid.g() as f32 / 255.0, solid.b() as f32 / 255.0, 1.0];
                if ui.color_edit_button_rgba_unmultiplied(&mut color_array).changed() {
                    state.theme.solid_color = Color32::from_rgb((color_array[0] * 255.0) as u8, (color_array[1] * 255.0) as u8, (color_array[2] * 255.0) as u8);
                    state.save();
                }
            }
            ui.add_space(20.0);
            ui.horizontal(|ui| {
                if ui.button(RichText::new("Apply Theme").color(Color32::WHITE).size(12.0)).clicked() { state.theme_modal_open = false; }
                if ui.button(RichText::new("Reset").color(Color32::WHITE).size(12.0)).clicked() { state.theme = ThemeConfig::default(); }
                if ui.button(RichText::new("✕").color(Color32::WHITE).size(14.0)).clicked() { state.theme_modal_open = false; }
            });
        });
}

// =============================================================================
// WGPU RENDER STATE
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
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor { backends: wgpu::Backends::all(), dx12_shader_compiler: Default::default(), flags: wgpu::InstanceFlags::empty(), gles_minor_version: wgpu::Gles3MinorVersion::Automatic });
        let surface = instance.create_surface(window).map_err(|e| format!("Failed to create surface: {}", e))?;
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions { power_preference: wgpu::PowerPreference::LowPower, compatible_surface: Some(&surface), force_fallback_adapter: false }).await.ok_or_else(|| "Failed to request adapter".to_string())?;
        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor { label: Some("device"), required_features: wgpu::Features::empty(), required_limits: adapter.limits(), memory_hints: wgpu::MemoryHints::default() }, None).await.map_err(|e| format!("Failed to request device: {}", e))?;
        let size = window.inner_size();
        let capabilities = surface.get_capabilities(&adapter);
        let format = capabilities.formats.first().copied().unwrap_or(wgpu::TextureFormat::Bgra8UnormSrgb);
        let surface_config = wgpu::SurfaceConfiguration { usage: wgpu::TextureUsages::RENDER_ATTACHMENT, format, width: size.width, height: size.height, present_mode: wgpu::PresentMode::Fifo, alpha_mode: wgpu::CompositeAlphaMode::Auto, view_formats: vec![], desired_maximum_frame_latency: 2 };
        surface.configure(&device, &surface_config);
        let renderer = egui_wgpu::Renderer::new(&device, format, None, 1, false);
        Ok(Self { device, queue, surface, surface_config, renderer })
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
    if unsafe { GetCursorPos(&mut pt) }.is_ok() { Some((pt.x, pt.y)) } else { None }
}

#[cfg(not(windows))]
fn get_global_cursor() -> Option<(i32, i32)> { None }

fn log_to_file(msg: &str) {
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open("debug.log") { let _ = writeln!(file, "{}", msg); }
}

#[cfg(windows)]
fn set_window_topmost(hwnd: HWND) {
    unsafe { let _ = SetWindowPos(hwnd, HWND_TOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_SHOWWINDOW); }
}

#[cfg(not(windows))]
fn set_window_topmost() {}

fn main() {
    println!("==========================================");
    println!("  Daily Motivation - Pure Rust GUI");
    println!("  Built with winit + wgpu + egui");
    println!("==========================================");
    std::io::Write::flush(&mut std::io::stdout()).ok();

    log_to_file("Starting application");
    let event_loop = EventLoop::new().unwrap();

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

    let _ = event_loop.run_app(&mut app_runner);
    log_to_file("Event loop exited");
}

fn setup_fonts(ctx: &Context) {
    let mut fonts = egui::FontDefinitions::default();
    let font_paths = [
        "C:\\Windows\\Fonts\\Nirmala.ttc", "C:\\Windows\\Fonts\\Vrinda.ttf",
        "C:\\Windows\\Fonts\\Siyamrupali.ttf", "C:\\Windows\\Fonts\\ShonarBangla.ttf",
        "C:\\Windows\\Fonts\\Shonar.ttf", "C:\\Windows\\Fonts\\NotoSansBengali-Regular.ttf",
        "C:\\Windows\\Fonts\\arialuni.ttf", "NotoSansBengali-Regular.ttf",
        "assets/NotoSansBengali-Regular.ttf",
    ];
    let mut loaded = false;
    for path in font_paths {
        if let Ok(data) = std::fs::read(path) {
            fonts.font_data.insert("bengali".to_owned(), egui::FontData::from_owned(data));
            if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Proportional) { family.insert(0, "bengali".to_owned()); }
            if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Monospace) { family.insert(0, "bengali".to_owned()); }
            log_to_file(&format!("Loaded Bengali font from: {}", path));
            loaded = true;
            break;
        }
    }
    if !loaded { log_to_file("WARNING: No Bengali fonts found."); }
    fonts.font_data.insert("nerdfonts".to_owned(), egui::FontData::from_static(include_bytes!("../assets/nerdfonts_regular.ttf")));
    if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Proportional) { family.push("nerdfonts".to_owned()); }
    ctx.set_fonts(fonts);
}

fn contains_bengali(text: &str) -> bool {
    text.chars().any(|c| matches!(c, '\u{0980}'..='\u{09FF}'))
}

fn render_shaped_text(
    ctx: &Context,
    font_system: &mut cosmic_text::FontSystem,
    swash_cache: &mut cosmic_text::SwashCache,
    text: &str,
    font_size: f32,
    color: Color32,
    tex_cache: &mut HashMap<u64, egui::TextureHandle>,
) -> Option<(egui::TextureId, Vec2)> {
    if text.is_empty() { return None; }
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    text.hash(&mut hasher); font_size.to_bits().hash(&mut hasher); color.to_array().hash(&mut hasher);
    let cache_key = hasher.finish();
    if let Some(handle) = tex_cache.get(&cache_key) { let size = handle.size(); return Some((handle.id(), Vec2::new(size[0] as f32, size[1] as f32))); }
    let metrics = cosmic_text::Metrics::new(font_size, font_size * 1.3);
    let mut buffer = cosmic_text::Buffer::new(font_system, metrics);
    buffer.set_size(font_system, Some(2000.0), None);
    let attrs = cosmic_text::Attrs::new().family(cosmic_text::Family::Name("Nirmala UI"));
    buffer.set_text(font_system, text, attrs, cosmic_text::Shaping::Advanced);
    buffer.shape_until_scroll(font_system, false);
    let mut max_width: f32 = 0.0; let mut total_height: f32 = 0.0;
    for run in buffer.layout_runs() { max_width = max_width.max(run.line_w); total_height += run.line_height; }
    if max_width <= 0.0 || total_height <= 0.0 { return None; }
    let width = (max_width.ceil() as usize).max(1); let height = (total_height.ceil() as usize).max(1);
    let mut pixels = vec![Color32::TRANSPARENT; width * height];
    let text_color = cosmic_text::Color::rgba(color.r(), color.g(), color.b(), color.a());
    buffer.draw(font_system, swash_cache, text_color, |x, y, _w, _h, drawn_color| {
        let px = x as usize; let py = y as usize;
        if px < width && py < height && x >= 0 && y >= 0 {
            let alpha = drawn_color.a();
            if alpha > 0 { let idx = py * width + px; pixels[idx] = Color32::from_rgba_premultiplied(drawn_color.r(), drawn_color.g(), drawn_color.b(), alpha); }
        }
    });
    let image = egui::ColorImage { size: [width, height], pixels };
    let texture = ctx.load_texture(format!("shaped_{}", cache_key), image, egui::TextureOptions::LINEAR);
    let size = Vec2::new(width as f32, height as f32);
    let tex_id = texture.id();
    tex_cache.insert(cache_key, texture);
    Some((tex_id, size))
}

use winit::application::ApplicationHandler;
use winit::event_loop::ActiveEventLoop;

struct AppRunner {
    window: Option<&'static Window>,
    render_state: Option<WgpuRenderState<'static>>,
    app_state: Option<AppState>,
    egui_ctx: Option<Context>,
    egui_state: Option<egui_winit::State>,
    font_system: Option<cosmic_text::FontSystem>,
    swash_cache: Option<cosmic_text::SwashCache>,
    shaped_text_textures: HashMap<u64, egui::TextureHandle>,
    should_close: bool,
}

impl ApplicationHandler for AppRunner {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() { return; }
        log_to_file("resumed() called - creating window");
        match event_loop.create_window(
            Window::default_attributes()
                .with_title("Daily Motivation")
                .with_inner_size(LogicalSize::new(DEFAULT_WINDOW_SIZE.0 as f64, DEFAULT_WINDOW_SIZE.1 as f64))
                .with_min_inner_size(LogicalSize::new(MIN_WINDOW_SIZE.0 as f64, MIN_WINDOW_SIZE.1 as f64))
                .with_decorations(false).with_resizable(true).with_transparent(true).with_visible(false),
        ) {
            Ok(window) => {
                let window = Box::leak(Box::new(window));
                #[cfg(windows)] {
                    use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
                    if let Ok(handle) = window.window_handle() {
                        if let RawWindowHandle::Win32(win32_handle) = handle.as_raw() {
                            let hwnd = HWND(win32_handle.hwnd.get() as *mut _);
                            set_window_topmost(hwnd);
                        }
                    }
                }
                self.window = Some(window);
                match pollster::block_on(WgpuRenderState::new(window)) {
                    Ok(render_state) => {
                        let app_state = AppState::default();
                        let egui_ctx = Context::default();
                        let mut style = egui::Style::default();
                        style.visuals = egui::Visuals::dark();
                        style.visuals.window_fill = CANVAS_BG;
                        style.visuals.panel_fill = CONTROL_PANEL_BG;
                        let mut visuals = style.visuals.clone();
                        visuals.widgets.hovered.bg_fill = Color32::from_rgb(80, 80, 90);
                        visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, Color32::WHITE.gamma_multiply(0.5));
                        visuals.widgets.active.bg_fill = Color32::from_rgb(100, 100, 110);
                        visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, Color32::from_rgba_unmultiplied(190, 230, 255, 255));
                        visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, Color32::WHITE);
                        visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0, NEON_CYAN);
                        visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, NEON_CYAN);
                        style.visuals = visuals;
                        egui_ctx.set_style(style);
                        let egui_state = egui_winit::State::new(egui_ctx.clone(), egui::ViewportId::ROOT, window, None, None, None);
                        self.render_state = Some(render_state);
                        self.app_state = Some(app_state);
                        self.egui_ctx = Some(egui_ctx.clone());
                        self.egui_state = Some(egui_state);
                        setup_fonts(&egui_ctx);
                        window.set_visible(true);
                        log_to_file("Render state stored in AppRunner");
                    }
                    Err(e) => { eprintln!("Render state initialization failed: {}", e); event_loop.exit(); }
                }
            }
            Err(e) => { eprintln!("Failed to create window: {}", e); event_loop.exit(); }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: winit::window::WindowId, event: WindowEvent) {
        if let Some(window) = self.window {
            if let Some(egui_state) = self.egui_state.as_mut() { let _ = egui_state.on_window_event(window, &event); }
            match event {
                WindowEvent::CloseRequested => { event_loop.exit(); }
                WindowEvent::Resized(size) => { if let Some(render_state) = self.render_state.as_mut() { render_state.resize(size); } }
                WindowEvent::RedrawRequested => { self.render(&window); }
                _ => {}
            }
        }
        if let Some(app_state) = self.app_state.as_mut() {
            match event {
                WindowEvent::CursorMoved { .. } | WindowEvent::MouseInput { .. } | WindowEvent::KeyboardInput { .. } => {
                    app_state.last_interaction = Instant::now();
                    if let WindowEvent::KeyboardInput { event, .. } = event {
                        if event.state == winit::event::ElementState::Pressed {
                            if let winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Space) = event.physical_key {
                                app_state.active_animation = AppAnimation::None;
                                if let Some(window) = self.window {
                                    if let Ok(handle) = window.window_handle() {
                                        if let winit::raw_window_handle::RawWindowHandle::Win32(win32) = handle.as_raw() {
                                            let hwnd = HWND(win32.hwnd.get() as _);
                                            unsafe { let _ = SetLayeredWindowAttributes(hwnd, None, 255, LWA_ALPHA); }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    self.window.as_ref().map(|w| w.request_redraw());
                }
                _ => {}
            }
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if self.should_close { event_loop.exit(); return; }
        if let Some(window) = self.window { self.render(&window); }
        if self.should_close { event_loop.exit(); return; }
        let sleep_ms = if let Some(ctx) = self.egui_ctx.as_ref() { if ctx.has_requested_repaint() { 16 } else { 100 } } else { 16 };
        thread::sleep(Duration::from_millis(sleep_ms));
    }
}

impl AppRunner {
    fn render(&mut self, window: &Window) {
        let mut font_system = self.font_system.take();
        let mut swash_cache = self.swash_cache.take();
        let mut tex_cache = std::mem::take(&mut self.shaped_text_textures);

        let (app_state, egui_ctx, egui_state, render_state) = match (self.app_state.as_mut(), self.egui_ctx.as_mut(), self.egui_state.as_mut(), self.render_state.as_mut()) {
            (Some(state), Some(ctx), Some(est), Some(rst)) => (state, ctx, est, rst),
            _ => { self.font_system = font_system; self.swash_cache = swash_cache; self.shaped_text_textures = tex_cache; return; }
        };

        let mut raw_input = egui_state.take_egui_input(window);
        let scale = window.scale_factor() as f32;
        let content_w = window.inner_size().width as f32 / scale;
        let content_h = window.inner_size().height as f32 / scale;
        let content_rect = Rect::from_min_max(Pos2::new(0.0, TITLE_BAR_HEIGHT), Pos2::new(content_w, content_h));
        transform_raw_input_for_rotation_scale(&mut raw_input, content_rect, app_state.current_rotation_angle, app_state.current_scale);

        let full_output = egui_ctx.run(raw_input, |ctx| {
            if ctx.is_using_pointer() || ctx.input(|i| i.pointer.any_down() || !i.events.is_empty()) { app_state.last_interaction = Instant::now(); }

            let mut is_resizing = false;
            if let Some((dir, start_cx, start_cy, start_wx, start_wy, start_w, start_h)) = app_state.manual_resize_start {
                is_resizing = true;
                if ctx.input(|i| i.pointer.primary_down()) {
                    if let Some((cx, cy)) = get_global_cursor() {
                        let dx = cx - start_cx; let dy = cy - start_cy;
                        let mut new_w = start_w as i32; let mut new_h = start_h as i32;
                        let mut new_x = start_wx; let mut new_y = start_wy;
                        use winit::window::ResizeDirection;
                        match dir {
                            ResizeDirection::East => new_w += dx,
                            ResizeDirection::West => { new_w -= dx; new_x += dx; }
                            ResizeDirection::South => new_h += dy,
                            ResizeDirection::North => { new_h -= dy; new_y += dy; }
                            ResizeDirection::SouthEast => { new_w += dx; new_h += dy; }
                            ResizeDirection::SouthWest => { new_w -= dx; new_x += dx; new_h += dy; }
                            ResizeDirection::NorthEast => { new_w += dx; new_h -= dy; new_y += dy; }
                            ResizeDirection::NorthWest => { new_w -= dx; new_x += dx; new_h -= dy; new_y += dy; }
                        }
                        let new_w = new_w.max(0) as u32; let new_h = new_h.max(0) as u32;
                        window.set_outer_position(winit::dpi::PhysicalPosition::new(new_x, new_y));
                        let _ = window.request_inner_size(winit::dpi::PhysicalSize::new(new_w, new_h));
                    }
                } else { app_state.manual_resize_start = None; }
            }

            let border = 8.0; let screen_rect = ctx.screen_rect();
            if !is_resizing {
                if let Some(pos) = ctx.input(|i| i.pointer.hover_pos()) {
                    let left = pos.x < border; let right = pos.x > screen_rect.max.x - border;
                    let top = pos.y < border; let bottom = pos.y > screen_rect.max.y - border;
                    if left || right || top || bottom {
                        if top && left { ctx.set_cursor_icon(egui::CursorIcon::ResizeNwSe); }
                        else if top && right { ctx.set_cursor_icon(egui::CursorIcon::ResizeNeSw); }
                        else if bottom && left { ctx.set_cursor_icon(egui::CursorIcon::ResizeNeSw); }
                        else if bottom && right { ctx.set_cursor_icon(egui::CursorIcon::ResizeNwSe); }
                        else if top || bottom { ctx.set_cursor_icon(egui::CursorIcon::ResizeVertical); }
                        else if left || right { ctx.set_cursor_icon(egui::CursorIcon::ResizeHorizontal); }
                        if ctx.input(|i| i.pointer.primary_pressed()) {
                            use winit::window::ResizeDirection;
                            let dir = if top && left { ResizeDirection::NorthWest } else if top && right { ResizeDirection::NorthEast } else if bottom && left { ResizeDirection::SouthWest } else if bottom && right { ResizeDirection::SouthEast } else if top { ResizeDirection::North } else if bottom { ResizeDirection::South } else if left { ResizeDirection::West } else { ResizeDirection::East };
                            if let (Some((cx, cy)), Ok(wpos)) = (get_global_cursor(), window.outer_position()) {
                                let size = window.inner_size();
                                app_state.manual_resize_start = Some((dir, cx, cy, wpos.x, wpos.y, size.width, size.height));
                            } else { let _ = window.drag_resize_window(dir); }
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
                                let (pos_x, pos_y) = if let Ok(pos) = window.outer_position() { (pos.x, pos.y) } else { (0, 0) };
                                #[cfg(windows)] {
                                    use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
                                    let mut main_hwnd_isize = 0isize;
                                    if let Ok(handle) = window.window_handle() { if let RawWindowHandle::Win32(win32) = handle.as_raw() { main_hwnd_isize = win32.hwnd.get() as isize; } }
                                    let dev_path = "background/target/release/quantum_logo.exe";
                                    let rel_path = "quantum_logo.exe";
                                    let child_res = if std::path::Path::new(rel_path).exists() {
                                        std::process::Command::new(rel_path).args([&size.width.to_string(), &size.height.to_string(), &pos_x.to_string(), &pos_y.to_string(), &main_hwnd_isize.to_string()]).spawn()
                                    } else if std::path::Path::new(dev_path).exists() {
                                        std::process::Command::new(dev_path).args([&size.width.to_string(), &size.height.to_string(), &pos_x.to_string(), &pos_y.to_string(), &main_hwnd_isize.to_string()]).spawn()
                                    } else {
                                        std::process::Command::new("cargo").args(["run", "--release", "--manifest-path", "background/Cargo.toml", "--", &size.width.to_string(), &size.height.to_string(), &pos_x.to_string(), &pos_y.to_string(), &main_hwnd_isize.to_string()]).spawn()
                                    };
                                    if let Ok(child) = child_res { app_state.bg_process = Some(child); app_state.bg_hwnd = None; }
                                }
                                #[cfg(not(windows))] {
                                    if let Ok(child) = std::process::Command::new("cargo").args(["run", "--release", "--manifest-path", "background/Cargo.toml", "--", &size.width.to_string(), &size.height.to_string(), &pos_x.to_string(), &pos_y.to_string(), "0"]).spawn() {
                                        app_state.bg_process = Some(child); app_state.bg_hwnd = None;
                                    }
                                }
                            }
                        } else {
                            if let Some(mut child) = app_state.bg_process.take() { let _ = child.kill(); let _ = child.wait(); }
                        }
                    }
                    TitleBarAction::ExportClicked => {
                        if let Ok(json) = serde_json::to_string_pretty(&app_state.quotes) {
                            if let Ok(mut file) = OpenOptions::new().create(true).write(true).truncate(true).open("quotes_export.json") { let _ = file.write_all(json.as_bytes()); }
                        }
                    }
                    TitleBarAction::ZoomIn => { app_state.title_bar_state.zoom_level = (app_state.title_bar_state.zoom_level + 0.1).min(2.0); }
                    TitleBarAction::ZoomOut => { app_state.title_bar_state.zoom_level = (app_state.title_bar_state.zoom_level - 0.1).max(0.5); }
                    TitleBarAction::TogglePanel => { app_state.title_bar_state.control_panel_visible = !app_state.title_bar_state.control_panel_visible; }
                    TitleBarAction::MinimizeClicked => { window.set_minimized(true); }
                    TitleBarAction::MaximizeClicked => { window.set_maximized(!window.is_maximized()); }
                    TitleBarAction::CloseClicked => { self.should_close = true; }
                    TitleBarAction::HideHeader => { app_state.title_bar_state.header_visible = false; }
                    TitleBarAction::ShowHeader => { app_state.title_bar_state.header_visible = true; }
                    TitleBarAction::AnimateClicked => { app_state.active_animation = if app_state.active_animation == AppAnimation::Bounce { AppAnimation::None } else { AppAnimation::Bounce }; }
                    TitleBarAction::PlayBounce => {
                        if app_state.active_animation == AppAnimation::None { if let Ok(pos) = window.outer_position() { app_state.base_pos = Some((pos.x, pos.y)); } }
                        app_state.active_animation = if app_state.active_animation == AppAnimation::Bounce { AppAnimation::None } else { AppAnimation::Bounce };
                    }
                    TitleBarAction::PlayShake => {
                        if app_state.active_animation == AppAnimation::None { if let Ok(pos) = window.outer_position() { app_state.base_pos = Some((pos.x, pos.y)); } }
                        app_state.active_animation = if app_state.active_animation == AppAnimation::Shake { AppAnimation::None } else { AppAnimation::Shake };
                    }
                    TitleBarAction::PlayDance => {
                        if app_state.active_animation == AppAnimation::None { if let Ok(pos) = window.outer_position() { app_state.base_pos = Some((pos.x, pos.y)); } }
                        app_state.active_animation = if app_state.active_animation == AppAnimation::Dance { AppAnimation::None } else { AppAnimation::Dance };
                    }
                    TitleBarAction::PlayRotate => { app_state.rotation = app_state.rotation.wrapping_add(1); app_state.target_rotation_angle = app_state.rotation as f32 * std::f32::consts::FRAC_PI_2; }
                    TitleBarAction::PlayDissolve => {
                        if app_state.active_animation == AppAnimation::None { if let Ok(pos) = window.outer_position() { app_state.base_pos = Some((pos.x, pos.y)); } }
                        app_state.active_animation = if app_state.active_animation == AppAnimation::Dissolve { AppAnimation::None } else { AppAnimation::Dissolve };
                        if app_state.active_animation == AppAnimation::None {
                            if let Ok(handle) = window.window_handle() { if let winit::raw_window_handle::RawWindowHandle::Win32(win32) = handle.as_raw() { let hwnd = HWND(win32.hwnd.get() as _); unsafe { let _ = SetLayeredWindowAttributes(hwnd, None, 255, LWA_ALPHA); } } }
                        }
                    }
                    TitleBarAction::PlayFly => {
                        if app_state.active_animation == AppAnimation::None { if let Ok(pos) = window.outer_position() { app_state.base_pos = Some((pos.x, pos.y)); } }
                        app_state.active_animation = if app_state.active_animation == AppAnimation::Fly { AppAnimation::None } else { AppAnimation::Fly };
                    }
                    TitleBarAction::StopAnimations => {
                        app_state.active_animation = AppAnimation::None;
                        if let Ok(handle) = window.window_handle() { if let winit::raw_window_handle::RawWindowHandle::Win32(win32) = handle.as_raw() { let hwnd = HWND(win32.hwnd.get() as _); unsafe { let _ = SetLayeredWindowAttributes(hwnd, None, 255, LWA_ALPHA); } } }
                        if let Some((x, y)) = app_state.base_pos { window.set_outer_position(winit::dpi::PhysicalPosition::new(x, y)); }
                        app_state.base_pos = None;
                    }
                }
            }

            // Animation Engine
            if app_state.active_animation != AppAnimation::None {
                if let (Ok(pos), Some(monitor)) = (window.outer_position(), window.current_monitor()) {
                    let size = window.outer_size(); let monitor_size = monitor.size();
                    app_state.anim_progress += 0.016;
                    if app_state.base_pos.is_none() { app_state.base_pos = Some((pos.x, pos.y)); }
                    let (base_x, base_y) = app_state.base_pos.unwrap();
                    match app_state.active_animation {
                        AppAnimation::Bounce => {
                            let mut new_x = pos.x as f32 + app_state.bounce_vel_x; let mut new_y = pos.y as f32 + app_state.bounce_vel_y;
                            if new_x < 0.0 { new_x = 0.0; app_state.bounce_vel_x *= -1.0; } else if new_x + size.width as f32 > monitor_size.width as f32 { new_x = monitor_size.width as f32 - size.width as f32; app_state.bounce_vel_x *= -1.0; }
                            if new_y < 0.0 { new_y = 0.0; app_state.bounce_vel_y *= -1.0; } else if new_y + size.height as f32 > monitor_size.height as f32 { new_y = monitor_size.height as f32 - size.height as f32; app_state.bounce_vel_y *= -1.0; }
                            window.set_outer_position(winit::dpi::PhysicalPosition::new(new_x as i32, new_y as i32));
                            app_state.base_pos = Some((new_x as i32, new_y as i32));
                        }
                        AppAnimation::Shake => { let intensity = 12.0; let offset_x = (app_state.anim_progress * 130.0).sin() * intensity; let offset_y = (app_state.anim_progress * 115.0).cos() * intensity; window.set_outer_position(winit::dpi::PhysicalPosition::new(base_x + offset_x as i32, base_y + offset_y as i32)); }
                        AppAnimation::Dance => { let radius = 70.0; let offset_x = (app_state.anim_progress * 4.0).sin() * radius; let offset_y = (app_state.anim_progress * 2.5).cos() * radius; window.set_outer_position(winit::dpi::PhysicalPosition::new(base_x + offset_x as i32, base_y + offset_y as i32)); }
                        AppAnimation::Rotate => { if app_state.anim_progress > 2.5 { app_state.anim_progress = 0.0; actions.push(TitleBarAction::PlayRotate); } }
                        AppAnimation::Dissolve => {
                            if let Ok(handle) = window.window_handle() { if let winit::raw_window_handle::RawWindowHandle::Win32(win32) = handle.as_raw() {
                                let hwnd = HWND(win32.hwnd.get() as _);
                                let opacity = 0.4 + 0.6 * (app_state.anim_progress * 2.5).cos().abs();
                                unsafe { let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE); if (ex_style & WS_EX_LAYERED.0 as i32) == 0 { let _ = SetWindowLongW(hwnd, GWL_EXSTYLE, ex_style | WS_EX_LAYERED.0 as i32); } let _ = SetLayeredWindowAttributes(hwnd, None, (opacity * 255.0) as u8, LWA_ALPHA); }
                            } }
                        }
                        AppAnimation::Fly => { let speed = 12.0; let mut new_x = pos.x as f32 + speed; let offset_y = (app_state.anim_progress * 2.0).sin() * 150.0; if new_x > monitor_size.width as f32 { new_x = -(size.width as f32); } window.set_outer_position(winit::dpi::PhysicalPosition::new(new_x as i32, (monitor_size.height as f32 / 2.0 + offset_y) as i32)); }
                        _ => {}
                    }
                    window.request_redraw();
                }
            } else {
                if app_state.base_pos.is_some() {
                    if let Ok(handle) = window.window_handle() { if let winit::raw_window_handle::RawWindowHandle::Win32(win32) = handle.as_raw() { let hwnd = HWND(win32.hwnd.get() as _); unsafe { let _ = SetLayeredWindowAttributes(hwnd, None, 255, LWA_ALPHA); } } }
                    if matches!(app_state.active_animation, AppAnimation::Shake | AppAnimation::Dance) { if let Some((x, y)) = app_state.base_pos { window.set_outer_position(winit::dpi::PhysicalPosition::new(x, y)); } }
                    app_state.base_pos = None; app_state.anim_progress = 0.0;
                }
            }

            // Auto-rotate (skip hidden quotes)
            if app_state.rotation_enabled && app_state.last_rotation.elapsed() >= app_state.rotation_interval && !app_state.quotes.is_empty() {
                // Find next visible quote
                let next = app_state.next_visible_index(app_state.current_quote_index);
                if let Some(idx) = next {
                    app_state.current_quote_index = idx;
                    app_state.last_rotation = Instant::now();
                } else {
                    // All hidden, just reset timer
                    app_state.last_rotation = Instant::now();
                }
            }

            let mut shaper = match (font_system.as_mut(), swash_cache.as_mut()) {
                (Some(fs), Some(sc)) => Some((fs, sc, &mut tex_cache)),
                _ => None,
            };

            // Smooth rotation/scale animation
            {
                let speed = 8.0_f32; let dt = 0.016_f32; let lerp = 1.0 - (-speed * dt).exp();
                app_state.current_rotation_angle += (app_state.target_rotation_angle - app_state.current_rotation_angle) * lerp;
                let angle = app_state.current_rotation_angle; let cos_a = angle.cos().abs(); let sin_a = angle.sin().abs();
                let w = content_rect.width(); let h = content_rect.height();
                let bounding_w = w * cos_a + h * sin_a; let bounding_h = w * sin_a + h * cos_a;
                let target_scale = (w / bounding_w).min(h / bounding_h).min(1.0);
                app_state.current_scale += (target_scale - app_state.current_scale) * lerp;
                if (app_state.current_rotation_angle - app_state.target_rotation_angle).abs() > 0.001 || (app_state.current_scale - target_scale).abs() > 0.001 { window.request_redraw(); }
            }

            #[cfg(windows)] {
                if let Ok(handle) = window.window_handle() { if let winit::raw_window_handle::RawWindowHandle::Win32(win32) = handle.as_raw() {
                    let hwnd = HWND(win32.hwnd.get() as _);
                    let mut property_name: Vec<u16> = "RotationState".encode_utf16().collect(); property_name.push(0);
                    let angle_bits = app_state.current_rotation_angle.to_bits();
                    unsafe { let _ = SetPropW(hwnd, windows::core::PCWSTR(property_name.as_ptr()), windows::Win32::Foundation::HANDLE(angle_bits as _)); }
                } }
            }

            render_main_content(ctx, app_state, &mut shaper);
            render_theme_modal(ctx, app_state);

            let float_actions = render_floating_buttons(ctx, app_state);
            for action in float_actions {
                match action {
                    TitleBarAction::TogglePanel => { app_state.title_bar_state.control_panel_visible = !app_state.title_bar_state.control_panel_visible; }
                    TitleBarAction::ShowHeader => { app_state.title_bar_state.header_visible = true; }
                    _ => {}
                }
            }
        });

        let scale = window.scale_factor() as f32;
        let content_w = window.inner_size().width as f32 / scale;
        let content_h = window.inner_size().height as f32 / scale;
        let content_rect = Rect::from_min_max(Pos2::new(0.0, TITLE_BAR_HEIGHT), Pos2::new(content_w, content_h));

        egui_state.handle_platform_output(window, full_output.platform_output);

        let shapes_to_tessellate = if app_state.current_rotation_angle.abs() > 0.0001 || (app_state.current_scale - 1.0).abs() > 0.0001 {
            transform_content_shapes(&full_output.shapes, content_rect, app_state.current_rotation_angle, app_state.current_scale)
        } else { full_output.shapes };
        let paint_jobs = egui_ctx.tessellate(shapes_to_tessellate, scale);

        let frame = match render_state.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(_) => { render_state.surface.configure(&render_state.device, &render_state.surface_config); return; }
        };
        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let screen_descriptor = egui_wgpu::ScreenDescriptor { size_in_pixels: [render_state.surface_config.width, render_state.surface_config.height], pixels_per_point: window.scale_factor() as f32 };
        let mut encoder = render_state.device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        for (id, image_delta) in &full_output.textures_delta.set { render_state.renderer.update_texture(&render_state.device, &render_state.queue, *id, image_delta); }
        render_state.renderer.update_buffers(&render_state.device, &render_state.queue, &mut encoder, &paint_jobs, &screen_descriptor);

        let bg_color = app_state.get_background_color();
        let clear_color = wgpu::Color { r: bg_color.r() as f64 / 255.0, g: bg_color.g() as f64 / 255.0, b: bg_color.b() as f64 / 255.0, a: bg_color.a() as f64 / 255.0 };
        {
            let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("egui_render"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment { view: &view, resolve_target: None, ops: wgpu::Operations { load: wgpu::LoadOp::Clear(clear_color), store: wgpu::StoreOp::Store } })],
                depth_stencil_attachment: None, timestamp_writes: None, occlusion_query_set: None,
            });
            let mut render_pass = render_pass.forget_lifetime();
            render_state.renderer.render(&mut render_pass, &paint_jobs, &screen_descriptor);
        }
        render_state.queue.submit(Some(encoder.finish()));
        frame.present();
        for id in &full_output.textures_delta.free { render_state.renderer.free_texture(id); }

        self.font_system = font_system;
        self.swash_cache = swash_cache;
        self.shaped_text_textures = tex_cache;
    }
}
