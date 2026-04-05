// =============================================================================
// card_header_widgets.rs
// ─────────────────────────────────────────────────────────────────────────────
// DROP-IN egui module for Task Card header row widgets:
//   [+] Plus button → creates unlimited nested sub-cards (95% → min 70% scale)
//   [clock₀][clock₁][clock₂] → three independent time badges per card
//
// Requires: egui 0.28+   (no extra deps beyond what main.rs already uses)
//
// Integration:
//   1. Add `mod card_header_widgets;` to main.rs
//   2. Add `CardHeaderState` to your Quote/Card struct
//   3. Call `draw_card_header_row(...)` inside your card rendering loop
//
// ─────────────────────────────────────────────────────────────────────────────
// AI DESIGN PROMPT (English) — for any AI image / UI generator:
// ─────────────────────────────────────────────────────────────────────────────
//
// "Design a futuristic task-card UI widget header bar with a dark transparent
//  background. On the far LEFT: a small square button (22×22 px) with a thin
//  rounded green border (#3CB450) and a bold crimson-red plus/cross symbol
//  centered inside it. The button pulses a soft green glow on hover. Immediately
//  to the right: THREE compact pill-shaped time-badge chips, each ~62×18 px,
//  thin green rounded border, displaying a time string like '12.10 PM' in dark
//  crimson text. Each badge has a distinct accent: first = deep red (deadline),
//  second = amber (sub-task), third = steel-blue (stopwatch). On hover each
//  badge glows in its accent color. The entire bar has no fill — only the
//  stroked borders on each widget are visible, giving a holographic circuit-
//  board feel. All elements sit on one horizontal row with 4 px gaps.
//  The font is monospace/condensed. Rounded corners = 5 px radius everywhere.
//  Aesthetic: biopunk meets quantum-computing terminal, year 50 000 CE."
//
// =============================================================================

use egui::{
    Color32, FontId, Pos2, Response, Rounding, Sense, Stroke, Ui, Vec2,
};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

// ─────────────────────────────────────────────────────────────────────────────
// PALETTE  (matches existing NEON constants in main.rs)
// ─────────────────────────────────────────────────────────────────────────────

const GREEN_BORDER: Color32 = Color32::from_rgb(60, 180, 80);  // existing app green

const RED_CROSS:   Color32 = Color32::from_rgb(178, 28, 28);   // crimson plus icon
const CLOCK_RED:   Color32 = Color32::from_rgb(165, 22, 22);   // deadline badge text
const CLOCK_AMBER: Color32 = Color32::from_rgb(185, 95, 15);   // sub-task badge text
const CLOCK_BLUE:  Color32 = Color32::from_rgb(28, 118, 185);  // stopwatch badge text

// ─────────────────────────────────────────────────────────────────────────────
// CLOCK MODE
// ─────────────────────────────────────────────────────────────────────────────

/// The three independent roles a clock badge can play.
#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub enum ClockMode {
    #[default]
    TaskDeadline,   // ① Fixed reminder / deadline — shows "HH.MM AM/PM"
    SubTaskTime,    // ② Scheduled sub-task time  — shows "HH.MM AM/PM"
    Stopwatch,      // ③ Live elapsed counter     — shows "MM:SS"
}

impl ClockMode {
    pub fn label(&self) -> &'static str {
        match self {
            Self::TaskDeadline => "Deadline",
            Self::SubTaskTime  => "Sub-task",
            Self::Stopwatch    => "Stopwatch",
        }
    }

    /// Cycle to the next mode.
    pub fn next(&self) -> Self {
        match self {
            Self::TaskDeadline => Self::SubTaskTime,
            Self::SubTaskTime  => Self::Stopwatch,
            Self::Stopwatch    => Self::TaskDeadline,
        }
    }

    /// Per-mode accent color (border glow + text).
    pub fn accent(&self) -> Color32 {
        match self {
            Self::TaskDeadline => CLOCK_RED,
            Self::SubTaskTime  => CLOCK_AMBER,
            Self::Stopwatch    => CLOCK_BLUE,
        }
    }

    pub fn border(&self) -> Color32 {
        // slightly brighter version of accent for the border stroke
        let [r, g, b, _] = self.accent().to_array();
        Color32::from_rgb(
            (r as u32 + 30).min(255) as u8,
            (g as u32 + 30).min(255) as u8,
            (b as u32 + 30).min(255) as u8,
        )
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// CLOCK BADGE STATE
// ─────────────────────────────────────────────────────────────────────────────

/// Persistent state for one clock badge.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClockBadge {
    pub mode:   ClockMode,
    /// Hour (0-23) and minute for Deadline / SubTask modes.
    pub hour:   u8,
    pub minute: u8,

    // ── Stopwatch internals (skip serializing runtime-only Instant) ──
    #[serde(skip)]
    pub sw_start:   Option<Instant>,
    pub sw_elapsed_ms: u64,      // persist elapsed milliseconds
    pub sw_running: bool,

    // ── Time-picker popup ─────────────────────────────────────────────
    #[serde(skip)]
    pub show_picker: bool,
    #[serde(skip)]
    pub pick_h: String,
    #[serde(skip)]
    pub pick_m: String,
}

impl Default for ClockBadge {
    fn default() -> Self {
        Self {
            mode:   ClockMode::TaskDeadline,
            hour:   12,
            minute: 10,
            sw_start:      None,
            sw_elapsed_ms: 0,
            sw_running:    false,
            show_picker:   false,
            pick_h: "12".into(),
            pick_m: "10".into(),
        }
    }
}

impl ClockBadge {
    pub fn new_subtask() -> Self {
        Self { mode: ClockMode::SubTaskTime, ..Default::default() }
    }

    pub fn new_stopwatch() -> Self {
        Self { mode: ClockMode::Stopwatch, hour: 0, minute: 0, ..Default::default() }
    }

    // ── Display text ──────────────────────────────────────────────────
    pub fn display_text(&self) -> String {
        match self.mode {
            ClockMode::TaskDeadline | ClockMode::SubTaskTime => {
                let ampm = if self.hour < 12 { "AM" } else { "PM" };
                let h12  = match self.hour % 12 { 0 => 12, h => h };
                format!("{:02}.{:02} {}", h12, self.minute, ampm)
            }
            ClockMode::Stopwatch => {
                let extra_ms = self.sw_start
                    .map(|s| s.elapsed().as_millis() as u64)
                    .unwrap_or(0);
                let total_ms = self.sw_elapsed_ms + extra_ms;
                let secs = (total_ms / 1000) % 60;
                let mins = (total_ms / 60_000) % 60;
                let hrs  =  total_ms / 3_600_000;
                if hrs > 0 {
                    format!("{:02}:{:02}:{:02}", hrs, mins, secs)
                } else {
                    format!("{:02}:{:02}", mins, secs)
                }
            }
        }
    }

    // ── Stopwatch controls ────────────────────────────────────────────
    pub fn sw_toggle(&mut self) {
        if self.sw_running {
            // Pause: accumulate elapsed
            if let Some(start) = self.sw_start.take() {
                self.sw_elapsed_ms += start.elapsed().as_millis() as u64;
            }
            self.sw_running = false;
        } else {
            // Start
            self.sw_start   = Some(Instant::now());
            self.sw_running = true;
        }
    }

    pub fn sw_reset(&mut self) {
        self.sw_running    = false;
        self.sw_start      = None;
        self.sw_elapsed_ms = 0;
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// CARD HEADER STATE  (attach this to your Quote / Card struct)
// ─────────────────────────────────────────────────────────────────────────────

/// Complete state for one card's header widgets.
/// Attach this to your existing Quote / TaskCard struct.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CardHeaderState {
    /// Three independent clock badges.
    pub clocks: [ClockBadge; 3],
}

impl Default for CardHeaderState {
    fn default() -> Self {
        Self {
            clocks: [
                ClockBadge::default(),           // ① Deadline (red)
                ClockBadge::new_subtask(),       // ② Sub-task (amber)
                ClockBadge::new_stopwatch(),     // ③ Stopwatch (blue)
            ],
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// SUB-CARD SCALE HELPER
// ─────────────────────────────────────────────────────────────────────────────

/// Width scale for a card at the given nesting depth.
///   depth 0 → 1.00   (100%, root card)
///   depth 1 → 0.95   (95%)
///   depth 2 → 0.9025 (≈90%)
///   depth 4 → 0.8145 (≈81%)
///   depth 6 → 0.7351 (≈74%)   ← approaches floor
///   depth 7+ → 0.70            (70% hard floor)
///
/// Apply this to `ui.set_max_width(parent_width * card_scale_at_depth(depth))`
/// before drawing each card, then indent `depth * INDENT_PX` pixels.
pub const INDENT_PX: f32 = 16.0;

pub fn card_scale_at_depth(depth: u8) -> f32 {
    0.95_f32.powi(depth as i32).max(0.70)
}

// ─────────────────────────────────────────────────────────────────────────────
// PLUS BUTTON WIDGET
// ─────────────────────────────────────────────────────────────────────────────

/// Draw the [+] button on the left of a card header.
///
/// Returns `true` on click → caller should call `add_sub_card(...)`.
///
/// `anim_time` — pass `ui.input(|i| i.time) as f32` for the pulse effect.
pub fn draw_plus_button(ui: &mut Ui, card_id: u64, anim_time: f32) -> bool {
    let size = Vec2::splat(22.0);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());

    if ui.is_rect_visible(rect) {
        let painter  = ui.painter();
        let hovered  = resp.hovered();

        // ── Animated outer glow (breathing pulse) ─────────────────────
        let pulse = ((anim_time * 1.8) + card_id as f32 * 0.3).sin() * 0.5 + 0.5; // 0.0..1.0
        let glow_alpha = if hovered { 70 } else { 28 };
        let glow_col = Color32::from_rgba_premultiplied(
            60, 210, 90,
            (glow_alpha as f32 * pulse) as u8,
        );
        painter.rect_filled(
            rect.expand(2.0 + pulse * 3.0),
            Rounding::same(8.0),
            glow_col,
        );

        // ── Border box ────────────────────────────────────────────────
        let border_col = if hovered {
            Color32::from_rgb(90, 230, 110)
        } else {
            GREEN_BORDER
        };
        let stroke_w = if hovered { 1.8 } else { 1.4 };
        painter.rect(
            rect,
            Rounding::same(5.0),
            Color32::TRANSPARENT,
            Stroke::new(stroke_w, border_col),
        );

        // ── Plus / Cross arms ─────────────────────────────────────────
        let cx    = rect.center();
        let arm   = 5.5_f32;
        let thick = if hovered { 2.4 } else { 1.9 };

        // Horizontal bar
        painter.line_segment(
            [Pos2::new(cx.x - arm, cx.y), Pos2::new(cx.x + arm, cx.y)],
            Stroke::new(thick, RED_CROSS),
        );
        // Vertical bar
        painter.line_segment(
            [Pos2::new(cx.x, cx.y - arm), Pos2::new(cx.x, cx.y + arm)],
            Stroke::new(thick, RED_CROSS),
        );

        // ── Hover tooltip ──────────────────────────────────────────────
        if hovered {
            resp.clone().on_hover_text("Add sub-card  (scales to 95%, min 70%)");
        }
    }

    resp.clicked()
}

// ─────────────────────────────────────────────────────────────────────────────
// CLOCK BADGE WIDGET
// ─────────────────────────────────────────────────────────────────────────────

/// Draw one clock badge.
///
/// **Left-click**:
///   - Deadline / Sub-task → open/close the mini time-picker
///   - Stopwatch           → start / pause
///
/// **Right-click**:
///   - Deadline / Sub-task → cycle to next mode
///   - Stopwatch           → reset counter
///
/// Returns `true` if state changed (so caller can mark as dirty / save).
pub fn draw_clock_badge(
    ui:        &mut Ui,
    badge:     &mut ClockBadge,
    badge_idx: usize,   // 0, 1 or 2 — used for unique popup IDs
    anim_time: f32,
) -> bool {
    let text    = badge.display_text();
    let accent  = badge.mode.accent();
    let font    = FontId::monospace(10.5);

    // ── Measure text to size the badge ────────────────────────────────
    let galley = ui.painter().layout_no_wrap(text.clone(), font.clone(), accent);
    let padding = Vec2::new(7.0, 4.0);
    let badge_sz = Vec2::new(
        (galley.size().x + padding.x * 2.0).max(64.0),
        (galley.size().y + padding.y * 2.0).max(19.0),
    );

    let (rect, resp) = ui.allocate_exact_size(badge_sz, Sense::click());
    let hovered      = resp.hovered();
    let l_click      = resp.clicked();
    let r_click      = resp.secondary_clicked();
    let mut changed  = false;

    // ── Right-click behaviour ──────────────────────────────────────────
    if r_click {
        match badge.mode {
            ClockMode::Stopwatch => {
                badge.sw_reset();
            }
            _ => {
                badge.mode = badge.mode.next();
                badge.show_picker = false;
            }
        }
        changed = true;
    }

    // ── Left-click behaviour ───────────────────────────────────────────
    if l_click {
        match badge.mode {
            ClockMode::Stopwatch => {
                badge.sw_toggle();
            }
            _ => {
                badge.show_picker = !badge.show_picker;
                if badge.show_picker {
                    badge.pick_h = format!("{:02}", badge.hour);
                    badge.pick_m = format!("{:02}", badge.minute);
                }
            }
        }
        changed = true;
    }

    // ── Painting ───────────────────────────────────────────────────────
    if ui.is_rect_visible(rect) {
        let painter = ui.painter();

        // Animated accent glow when stopwatch is ticking or hovered
        if badge.sw_running || hovered {
            let phase = (anim_time * 2.5 + badge_idx as f32 * 1.1).sin() * 0.5 + 0.5;
            let base_alpha: u8 = if badge.sw_running { 55 } else { 22 };
            let [r, g, b, _]   = accent.to_array();
            let glow = Color32::from_rgba_premultiplied(
                r, g, b,
                (base_alpha as f32 * phase) as u8,
            );
            painter.rect_filled(rect.expand(3.5), Rounding::same(9.0), glow);
        }

        // Badge border (uses mode-specific border color, brighter on hover)
        let border_col = if hovered {
            badge.mode.border()
        } else {
            // default green border matching the image exactly
            GREEN_BORDER
        };
        let bw = if hovered { 1.5 } else { 1.2 };
        painter.rect(rect, Rounding::same(5.0), Color32::TRANSPARENT,
                     Stroke::new(bw, border_col));

        // Running indicator: tiny pulsing dot for active stopwatch
        if badge.sw_running {
            let dot_phase = (anim_time * 4.0).sin() * 0.5 + 0.5;
            let dot_r     = 2.8 + dot_phase * 0.8;
            let dot_pos   = Pos2::new(rect.max.x - 6.0, rect.center().y);
            painter.circle_filled(dot_pos, dot_r,
                Color32::from_rgba_premultiplied(40, 200, 255, 200));
        }

        // Text
        let text_pos = Pos2::new(
            rect.min.x + padding.x,
            rect.center().y - galley.size().y / 2.0,
        );
        painter.galley(text_pos, galley, accent);

        // Tooltip
        if hovered {
            let tip = match badge.mode {
                ClockMode::TaskDeadline =>
                    "Task deadline │ Left: set time │ Right: change mode",
                ClockMode::SubTaskTime  =>
                    "Sub-task time │ Left: set time │ Right: change mode",
                ClockMode::Stopwatch    =>
                    "Stopwatch │ Left: start/pause │ Right: reset",
            };
            resp.clone().on_hover_text(tip);
        }
    }

    // ── Time-Picker Popup (Deadline & SubTask only) ────────────────────
    if badge.show_picker && badge.mode != ClockMode::Stopwatch {
        let popup_id  = ui.make_persistent_id(("clk_pick", badge_idx));
        let popup_pos = rect.left_bottom() + Vec2::new(0.0, 5.0);

        egui::Area::new(popup_id)
            .fixed_pos(popup_pos)
            .order(egui::Order::Tooltip)
            .interactable(true)
            .show(ui.ctx(), |ui| {
                egui::Frame::none()
                    .fill(Color32::from_rgb(14, 22, 14))
                    .stroke(Stroke::new(1.2, badge.mode.border()))
                    .rounding(Rounding::same(7.0))
                    .inner_margin(egui::Margin::symmetric(10.0, 8.0))
                    .show(ui, |ui| {
                        ui.set_min_width(130.0);

                        // Header label
                        ui.label(
                            egui::RichText::new(badge.mode.label())
                                .color(badge.mode.border())
                                .size(9.5)
                                .strong(),
                        );
                        ui.add_space(3.0);

                        // Hour : Minute + AM/PM
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 3.0;

                            ui.add(
                                egui::TextEdit::singleline(&mut badge.pick_h)
                                    .desired_width(30.0)
                                    .font(FontId::monospace(12.0))
                                    .hint_text("HH"),
                            );
                            ui.label(
                                egui::RichText::new(":")
                                    .color(GREEN_BORDER)
                                    .size(13.0),
                            );
                            ui.add(
                                egui::TextEdit::singleline(&mut badge.pick_m)
                                    .desired_width(30.0)
                                    .font(FontId::monospace(12.0))
                                    .hint_text("MM"),
                            );

                            // AM / PM toggle
                            let is_pm   = badge.hour >= 12;
                            let btn_lbl = if is_pm { "PM" } else { "AM" };
                            if ui.small_button(btn_lbl).clicked() {
                                badge.hour = if is_pm {
                                    badge.hour.saturating_sub(12)
                                } else {
                                    (badge.hour + 12).min(23)
                                };
                                changed = true;
                            }
                        });

                        ui.add_space(4.0);

                        // Confirm / Cancel
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 4.0;

                            if ui.button("✔ Set").clicked() {
                                if let Ok(h) = badge.pick_h.trim().parse::<u8>() {
                                    badge.hour = h.min(23);
                                }
                                if let Ok(m) = badge.pick_m.trim().parse::<u8>() {
                                    badge.minute = m.min(59);
                                }
                                badge.show_picker = false;
                                changed = true;
                            }
                            if ui.button("✖").clicked() {
                                badge.show_picker = false;
                                changed = true;
                            }
                        });
                    });
            });

        // Close picker on outside click
        let ptr = ui.input(|i| i.pointer.interact_pos());
        if ui.input(|i| i.pointer.any_click()) {
            if let Some(p) = ptr {
                if !rect.contains(p) {
                    badge.show_picker = false;
                    changed = true;
                }
            }
        }
    }

    // Continuously repaint while stopwatch ticks (every 500 ms is smooth enough)
    if badge.sw_running {
        ui.ctx().request_repaint_after(Duration::from_millis(500));
    }

    changed
}

// ─────────────────────────────────────────────────────────────────────────────
// FULL CARD HEADER ROW
// ─────────────────────────────────────────────────────────────────────────────

/// Draw the complete left header: `[+] [clock₀] [clock₁] [clock₂]`
///
/// Returns `(plus_clicked, badges_changed)`.
///
/// # Minimal integration example
/// ```rust
/// // In your card rendering loop:
/// let anim_time = ui.input(|i| i.time) as f32;
/// let (new_sub_card, _) = draw_card_header_row(
///     ui,
///     card.id,
///     &mut card.header,
///     anim_time,
/// );
/// if new_sub_card {
///     let depth = card.depth + 1;
///     card.children.push(TaskCard::new(next_id(), depth));
/// }
/// ```
pub fn draw_card_header_row(
    ui:        &mut Ui,
    card_id:   u64,
    header:    &mut CardHeaderState,
    anim_time: f32,
) -> (bool, bool) {
    let mut plus_clicked  = false;
    let mut badge_changed = false;

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;

        // [+] button
        if draw_plus_button(ui, card_id, anim_time) {
            plus_clicked = true;
        }

        // Three clock badges
        for (i, badge) in header.clocks.iter_mut().enumerate() {
            if draw_clock_badge(ui, badge, i, anim_time) {
                badge_changed = true;
            }
        }
    });

    (plus_clicked, badge_changed)
}

// ─────────────────────────────────────────────────────────────────────────────
// TASK CARD STRUCT  (lightweight — attach to your own card tree)
// ─────────────────────────────────────────────────────────────────────────────

/// Minimal task card with sub-card support.
/// Embed this inside your existing Quote/Card data model,
/// or use it standalone.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaskCard {
    pub id:       u64,
    pub depth:    u8,           // 0 = root, 1 = child, …
    pub content:  String,
    pub header:   CardHeaderState,
    pub children: Vec<TaskCard>,
}

impl TaskCard {
    pub fn new(id: u64, depth: u8) -> Self {
        Self {
            id,
            depth,
            content:  String::new(),
            header:   CardHeaderState::default(),
            children: Vec::new(),
        }
    }

    /// Add one sub-card directly under this card.
    /// `next_id` is a mutable counter the caller owns (e.g. `AppState.next_card_id`).
    pub fn add_sub_card(&mut self, next_id: &mut u64) {
        let child_depth = (self.depth + 1).min(20); // no practical limit, scale floors at 70 %
        self.children.push(TaskCard::new(*next_id, child_depth));
        *next_id += 1;
    }

    /// Recursively draw this card and all of its children.
    ///
    /// `available_width` — the pixel width of the *parent* container.
    ///   Child cards will use `available_width * card_scale_at_depth(self.depth)`.
    pub fn draw_recursive(
        &mut self,
        ui:             &mut Ui,
        available_width: f32,
        next_id:        &mut u64,
        anim_time:      f32,
    ) {
        let scale      = card_scale_at_depth(self.depth);
        let card_width = (available_width * scale).floor();

        // Indent left by depth * INDENT_PX
        ui.horizontal(|ui| {
            let indent = self.depth as f32 * INDENT_PX;
            ui.add_space(indent);

            ui.vertical(|ui| {
                ui.set_max_width(card_width);

                // ── Card outer frame ───────────────────────────────────
                let frame = egui::Frame::none()
                    .stroke(Stroke::new(1.4, GREEN_BORDER))
                    .rounding(Rounding::same(8.0))
                    .inner_margin(egui::Margin::symmetric(8.0, 6.0));

                frame.show(ui, |ui| {
                    ui.set_min_width(card_width - 16.0);

                    // ── Header row ─────────────────────────────────────
                    let (plus_clicked, _badge_changed) =
                        draw_card_header_row(ui, self.id, &mut self.header, anim_time);

                    if plus_clicked {
                        self.add_sub_card(next_id);
                    }

                    // ── Content area (multiline text) ──────────────────
                    ui.add_space(4.0);
                    let text_h = (60.0 * scale).max(30.0); // content area scales too
                    ui.add_sized(
                        Vec2::new(card_width - 32.0, text_h),
                        egui::TextEdit::multiline(&mut self.content)
                            .frame(false)
                            .hint_text("Write your note…"),
                    );
                });

                // ── Children recursively ───────────────────────────────
                for child in self.children.iter_mut() {
                    ui.add_space(4.0);
                    child.draw_recursive(ui, card_width, next_id, anim_time);
                }
            });
        });
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// INTEGRATION GUIDE (paste into your AppState / main render loop)
// ─────────────────────────────────────────────────────────────────────────────
//
// 1. Add to AppState:
//      pub root_cards: Vec<TaskCard>,
//      pub next_card_id: u64,
//
// 2. Init in AppState::default():
//      root_cards: vec![TaskCard::new(0, 0)],
//      next_card_id: 1,
//
// 3. In your central panel render function:
//      let anim_time = ui.input(|i| i.time) as f32;
//      let avail_w   = ui.available_width();
//      let next_id   = &mut state.next_card_id;
//
//      egui::ScrollArea::vertical().show(ui, |ui| {
//          for card in state.root_cards.iter_mut() {
//              card.draw_recursive(ui, avail_w, next_id, anim_time);
//              ui.add_space(8.0);
//          }
//      });
//
// 4. To add a brand-new root card from the title-bar [+] button:
//      let id = state.next_card_id;
//      state.next_card_id += 1;
//      state.root_cards.push(TaskCard::new(id, 0));
//
// ─────────────────────────────────────────────────────────────────────────────
