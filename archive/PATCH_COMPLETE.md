# Complete Patch — 4 Feature Updates

## Summary of Changes

| # | Feature | Location |
|---|---------|----------|
| 1 | Drag-to-reorder (▲▼ buttons per row) | `AppState`, `render_control_panel_contents` |
| 2 | Compact hide+delete buttons (1px gap, icon-only) | `render_control_panel_contents` list section |
| 3 | Hide = disappear from main display + skip in rotation | `AppState::next_quote`, `render_main_content` |
| 4 | Main display: glowing semi-rounded card per quote | `render_main_content` — new `render_quote_card` fn |

---

## STEP 1 — Add `drag_reorder_from` + `move_quote` + `next_visible_index` to AppState

### 1a. Add field to AppState struct (after `base_pos`)

```rust
// NEW field — add after `pub base_pos: Option<(i32, i32)>,`
pub drag_reorder_from: Option<usize>,
```

### 1b. Add to BOTH `Default` initializer blocks (the `if let Some(config)` arm AND the `else` arm)

```rust
drag_reorder_from: None,
```

### 1c. Add these 2 new methods to `impl AppState`

```rust
/// Find next non-hidden quote index starting AFTER `from`
pub fn next_visible_index(&self, from: usize) -> Option<usize> {
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

/// Move quote at `from` to position `to`, adjusting current_quote_index
pub fn move_quote(&mut self, from: usize, to: usize) {
    let len = self.quotes.len();
    if from == to || from >= len || to >= len { return; }
    let quote = self.quotes.remove(from);
    self.quotes.insert(to, quote);
    // Keep current_quote_index pointing to same quote
    if self.current_quote_index == from {
        self.current_quote_index = to;
    } else if from < to
        && self.current_quote_index > from
        && self.current_quote_index <= to
    {
        self.current_quote_index -= 1;
    } else if from > to
        && self.current_quote_index >= to
        && self.current_quote_index < from
    {
        self.current_quote_index += 1;
    }
    self.save();
}
```

### 1d. Replace `next_quote` to skip hidden quotes

```rust
// REPLACE the existing next_quote method:
pub fn next_quote(&mut self) {
    if !self.quotes.is_empty() {
        if let Some(idx) = self.next_visible_index(self.current_quote_index) {
            self.current_quote_index = idx;
        }
        self.last_rotation = Instant::now();
    }
}

// REPLACE the existing prev_quote method:
pub fn prev_quote(&mut self) {
    if !self.quotes.is_empty() {
        let len = self.quotes.len();
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
```

---

## STEP 2 — Replace the entire TEXT LIST section inside `render_control_panel_contents`

Find the block that starts with:
```rust
// ===== Quotes List Section =====
render_section(ui, &format!("TEXT LIST ({})", state.quotes.len()), ...
```

**Replace that entire `render_section(...)` call with:**

```rust
// ===== Quotes List Section =====
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
        let bg_color = if is_current {
            Color32::from_black_alpha(45)
        } else {
            Color32::from_black_alpha(20)
        };

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 2.0;

            // ── ▲▼ Reorder column (left side, stacked vertically) ──
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 1.0;

                let up_color = if idx > 0 {
                    NEON_CYAN.gamma_multiply(0.85)
                } else {
                    Color32::from_rgba_unmultiplied(100, 100, 120, 80)
                };
                let dn_color = if idx + 1 < n {
                    NEON_CYAN.gamma_multiply(0.85)
                } else {
                    Color32::from_rgba_unmultiplied(100, 100, 120, 80)
                };

                let up = ui.add_sized(
                    Vec2::new(14.0, 13.0),
                    egui::Button::new(
                        RichText::new("▲").size(8.0).color(up_color)
                    )
                    .fill(Color32::TRANSPARENT)
                    .frame(false),
                );
                if up.clicked() && idx > 0 {
                    move_from_to = Some((idx, idx - 1));
                }
                up.on_hover_text("Move up");

                let dn = ui.add_sized(
                    Vec2::new(14.0, 13.0),
                    egui::Button::new(
                        RichText::new("▼").size(8.0).color(dn_color)
                    )
                    .fill(Color32::TRANSPARENT)
                    .frame(false),
                );
                if dn.clicked() && idx + 1 < n {
                    move_from_to = Some((idx, idx + 1));
                }
                dn.on_hover_text("Move down");
            });

            // ── Main item box ──
            egui::Frame::none()
                .fill(bg_color)
                .inner_margin(egui::Margin { left: 6.0, right: 4.0, top: 5.0, bottom: 5.0 })
                .rounding(Rounding::same(5.0))
                .stroke(Stroke::new(
                    if is_current { 1.5 } else { 1.0 },
                    if is_current {
                        NEON_CYAN.gamma_multiply(0.55)
                    } else {
                        NEON_CYAN.gamma_multiply(0.18)
                    },
                ))
                .show(ui, |ui| {
                    // Width = total minus arrow column (≈16px) and outer spacing
                    let box_w = (list_box_internal_width - 18.0).max(10.0);
                    ui.set_max_width(box_w);
                    ui.set_width(box_w);

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // ── COMPACT action buttons — 1px gap, icon symbols only ──
                        ui.spacing_mut().item_spacing.x = 1.0; // was ~8px, now 1px → ~87% reduction

                        // Delete — ✕ symbol
                        let del = ui.add(
                            egui::Button::new(
                                RichText::new("✕")
                                    .color(NEON_ROSE.gamma_multiply(0.85))
                                    .size(10.5),
                            )
                            .fill(Color32::TRANSPARENT)
                            .min_size(Vec2::new(16.0, 16.0))
                            .frame(false),
                        );
                        if del.clicked() { to_delete = Some(idx); }
                        del.on_hover_text("Delete");

                        // Hide/Show — ◉ (visible) / ◎ (hidden) symbol
                        let (hide_sym, hide_tip, hide_col) = if is_hidden {
                            ("◎", "Show in display", NEON_SOLAR.gamma_multiply(0.9))
                        } else {
                            ("◉", "Hide from display",
                             Color32::from_rgba_unmultiplied(160, 200, 220, 180))
                        };
                        let hide = ui.add(
                            egui::Button::new(
                                RichText::new(hide_sym).color(hide_col).size(10.5),
                            )
                            .fill(Color32::TRANSPARENT)
                            .min_size(Vec2::new(16.0, 16.0))
                            .frame(false),
                        );
                        if hide.clicked() { to_toggle_hide = Some(idx); }
                        hide.on_hover_text(hide_tip);

                        // ── Text content area ──
                        ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
                            ui.vertical(|ui| {
                                let text_color = state.text_style.panel_text_color;
                                let alpha_mult = if is_hidden { 0.38 } else { 1.0 };
                                let display_color = text_color.linear_multiply(alpha_mult);

                                // Hidden badge
                                if is_hidden {
                                    ui.label(
                                        RichText::new("⊘ hidden")
                                            .size(8.0)
                                            .color(NEON_SOLAR.gamma_multiply(0.65)),
                                    );
                                }

                                // Main text line
                                let display_main = format!("{}. {}", idx + 1, &state.quotes[idx].main_text);
                                let mut clicked = false;

                                if contains_bengali(&state.quotes[idx].main_text) {
                                    if let Some((ref mut fs, ref mut sc, ref mut tc)) = shaper {
                                        if let Some((tex_id, size)) = render_shaped_text(
                                            ui.ctx(), fs, sc, &display_main, 13.0, display_color, tc,
                                        ) {
                                            let avail_w = ui.available_width();
                                            let mut dsz = size;
                                            let mut uv = egui::Rect::from_min_max(
                                                egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0),
                                            );
                                            let mut ellipsis = false;
                                            if size.x > avail_w {
                                                dsz.x = (avail_w - 12.0).max(1.0);
                                                uv.max.x = dsz.x / size.x;
                                                ellipsis = true;
                                            }
                                            clicked = ui.horizontal(|ui| {
                                                ui.spacing_mut().item_spacing.x = 0.0;
                                                let r = ui.add(
                                                    egui::Image::new(egui::load::SizedTexture::new(tex_id, dsz))
                                                        .uv(uv)
                                                        .sense(egui::Sense::click()),
                                                );
                                                if ellipsis {
                                                    ui.add(egui::Label::new(
                                                        RichText::new("…").color(display_color).size(13.0),
                                                    ));
                                                }
                                                r.clicked()
                                            }).inner;
                                        } else {
                                            clicked = ui.add(
                                                egui::Label::new(
                                                    RichText::new(&display_main)
                                                        .color(display_color).size(13.0),
                                                ).truncate(),
                                            ).clicked();
                                        }
                                    }
                                } else {
                                    clicked = ui.add(
                                        egui::Label::new(
                                            RichText::new(&display_main)
                                                .color(display_color).size(13.0),
                                        ).truncate(),
                                    ).clicked();
                                }

                                // Sub text line
                                let display_sub = format!("↳ {}", &state.quotes[idx].sub_text);
                                let sub_color = display_color.gamma_multiply(0.62);

                                if contains_bengali(&state.quotes[idx].sub_text) {
                                    if let Some((ref mut fs, ref mut sc, ref mut tc)) = shaper {
                                        if let Some((tex_id, size)) = render_shaped_text(
                                            ui.ctx(), fs, sc, &display_sub, 11.5, sub_color, tc,
                                        ) {
                                            let avail_w = ui.available_width();
                                            let mut dsz = size;
                                            let mut uv = egui::Rect::from_min_max(
                                                egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0),
                                            );
                                            let mut ellipsis = false;
                                            if size.x > avail_w {
                                                dsz.x = (avail_w - 12.0).max(1.0);
                                                uv.max.x = dsz.x / size.x;
                                                ellipsis = true;
                                            }
                                            ui.horizontal(|ui| {
                                                ui.spacing_mut().item_spacing.x = 0.0;
                                                ui.add(
                                                    egui::Image::new(egui::load::SizedTexture::new(tex_id, dsz))
                                                        .uv(uv),
                                                );
                                                if ellipsis {
                                                    ui.add(egui::Label::new(
                                                        RichText::new("…").color(sub_color).size(11.5),
                                                    ));
                                                }
                                            });
                                        } else {
                                            ui.add(egui::Label::new(
                                                RichText::new(&display_sub).color(sub_color).size(11.5),
                                            ).truncate());
                                        }
                                    }
                                } else {
                                    ui.add(egui::Label::new(
                                        RichText::new(&display_sub).color(sub_color).size(11.5),
                                    ).truncate());
                                }

                                if clicked { to_select = Some(idx); }
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
    if let Some(idx) = to_select {
        state.current_quote_index = idx;
        state.last_rotation = Instant::now();
    }
    if let Some(idx) = to_toggle_hide {
        state.quotes[idx].is_hidden = !state.quotes[idx].is_hidden;
        // If we just hid the currently displayed quote, jump to next visible
        if state.quotes[idx].is_hidden && state.current_quote_index == idx {
            if let Some(next) = state.next_visible_index(idx) {
                state.current_quote_index = next;
            }
        }
        state.save();
    }
});
```

---

## STEP 3 — Replace `render_main_content`'s central quote display

### 3a — Add this standalone function BEFORE `render_main_content`

```rust
/// Render a single motivational quote as a glowing semi-rounded card.
fn render_quote_card(
    ctx: &egui::Context,
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
) {
    let card_width = (ui.available_width() * 0.88).min(720.0).max(280.0);
    let main_size  = text_style.main_text_size * zoom_level;
    let sub_size   = text_style.sub_text_size  * zoom_level;

    // Reserve space; we'll paint the card background *after* to draw behind text
    let card_resp = ui.allocate_ui_with_layout(
        Vec2::new(card_width, 0.0),
        egui::Layout::top_down(egui::Align::Center),
        |ui| {
            ui.set_width(card_width);
            ui.add_space(28.0); // top padding

            // ── Main text ──
            let main_color = text_style.main_text_color;
            let mut did_shape = false;
            if contains_bengali(main_text) {
                if let Some((ref mut fs, ref mut sc, ref mut tc)) = shaper {
                    if let Some((tex_id, sz)) =
                        render_shaped_text(ctx, fs, sc, main_text, main_size, main_color, tc)
                    {
                        ui.add(
                            egui::Image::new(egui::load::SizedTexture::new(tex_id, sz))
                                .sense(egui::Sense::hover()),
                        );
                        did_shape = true;
                    }
                }
            }
            if !did_shape {
                ui.add(
                    egui::Label::new(
                        RichText::new(main_text)
                            .color(main_color)
                            .size(main_size)
                            .strong(),
                    )
                    .sense(egui::Sense::hover()),
                );
            }

            // ── Thin separator ──
            if !sub_text.is_empty() {
                ui.add_space(text_style.between_gap * 0.5);
                let sep_pos = ui.cursor().min;
                let sep_w   = card_width * 0.28;
                let sep_x   = sep_pos.x + card_width * 0.5;
                ui.painter().line_segment(
                    [
                        egui::pos2(sep_x - sep_w / 2.0, sep_pos.y),
                        egui::pos2(sep_x + sep_w / 2.0, sep_pos.y),
                    ],
                    Stroke::new(1.0, NEON_CYAN.gamma_multiply(0.28)),
                );
                ui.add_space(text_style.between_gap * 0.5);

                // ── Sub text ──
                let sub_color = text_style.sub_text_color;
                let mut did_shape_sub = false;
                if contains_bengali(sub_text) {
                    if let Some((ref mut fs, ref mut sc, ref mut tc)) = shaper {
                        if let Some((tex_id, sz)) =
                            render_shaped_text(ctx, fs, sc, sub_text, sub_size, sub_color, tc)
                        {
                            ui.add(
                                egui::Image::new(egui::load::SizedTexture::new(tex_id, sz))
                                    .sense(egui::Sense::hover()),
                            );
                            did_shape_sub = true;
                        }
                    }
                }
                if !did_shape_sub {
                    ui.add(
                        egui::Label::new(
                            RichText::new(sub_text).color(sub_color).size(sub_size),
                        )
                        .sense(egui::Sense::hover()),
                    );
                }
            }

            ui.add_space(28.0); // bottom padding
        },
    );

    let card_rect = card_resp.response.rect;
    let painter   = ui.painter();

    // ── Multi-layer outer glow ──
    for (expand, alpha) in [(20.0_f32, 0.025_f32), (12.0, 0.05), (5.0, 0.09)] {
        painter.rect_filled(
            card_rect.expand(expand),
            Rounding::same(20.0 + expand),
            NEON_CYAN.gamma_multiply(alpha),
        );
    }

    // ── Card fill (dark glass) ──
    painter.rect_filled(
        card_rect,
        Rounding::same(18.0),
        Color32::from_rgba_unmultiplied(6, 14, 30, 195),
    );

    // ── Inner top-edge glass rim ──
    painter.line_segment(
        [
            egui::pos2(card_rect.left() + 22.0, card_rect.top() + 1.5),
            egui::pos2(card_rect.right() - 22.0, card_rect.top() + 1.5),
        ],
        Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 28)),
    );

    // ── Outer border ──
    painter.rect_stroke(
        card_rect,
        Rounding::same(18.0),
        Stroke::new(1.5, NEON_CYAN.gamma_multiply(0.32)),
    );

    // ── Corner accent ticks ──
    let c  = 14.0; // tick length
    let cs = Stroke::new(2.0, NEON_CYAN.gamma_multiply(0.65));
    let tl = card_rect.left_top();
    let tr = card_rect.right_top();
    let bl = card_rect.left_bottom();
    let br = card_rect.right_bottom();
    // top-left
    painter.line_segment([egui::pos2(tl.x + 8.0, tl.y), egui::pos2(tl.x + 8.0 + c, tl.y)], cs);
    painter.line_segment([egui::pos2(tl.x, tl.y + 8.0), egui::pos2(tl.x, tl.y + 8.0 + c)], cs);
    // top-right
    painter.line_segment([egui::pos2(tr.x - 8.0, tr.y), egui::pos2(tr.x - 8.0 - c, tr.y)], cs);
    painter.line_segment([egui::pos2(tr.x, tr.y + 8.0), egui::pos2(tr.x, tr.y + 8.0 + c)], cs);
    // bottom-left
    painter.line_segment([egui::pos2(bl.x + 8.0, bl.y), egui::pos2(bl.x + 8.0 + c, bl.y)], cs);
    painter.line_segment([egui::pos2(bl.x, bl.y - 8.0), egui::pos2(bl.x, bl.y - 8.0 - c)], cs);
    // bottom-right
    painter.line_segment([egui::pos2(br.x - 8.0, br.y), egui::pos2(br.x - 8.0 - c, br.y)], cs);
    painter.line_segment([egui::pos2(br.x, br.y - 8.0), egui::pos2(br.x, br.y - 8.0 - c)], cs);
}
```

### 3b — Inside `render_main_content`, replace the ENTIRE `ScrollArea` contents block

Find the `egui::ScrollArea::vertical()...show(ui, |ui| { ui.vertical_centered(|ui| {` block and replace its inner content with:

```rust
egui::ScrollArea::vertical()
    .auto_shrink([false, false])
    .show(ui, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(60.0);

            let is_preview = !state.main_text_input.is_empty()
                || !state.sub_text_input.is_empty();

            if is_preview {
                // ── PREVIEW card (what you're about to add) ──
                let preview_main = if !state.main_text_input.is_empty() {
                    state.main_text_input.clone()
                } else {
                    "Type main text to preview…".to_string()
                };
                let preview_sub = state.sub_text_input.clone();
                render_quote_card(
                    ctx, ui, &preview_main, &preview_sub,
                    &state.text_style, state.title_bar_state.zoom_level,
                    shaper,
                );
                ui.add_space(40.0);

            } else if state.quotes.is_empty() {
                ui.label(
                    RichText::new("No quotes added yet!")
                        .color(Color32::GRAY).size(20.0),
                );
            } else {
                // ── SINGLE CURRENT QUOTE as glowing card ──
                // Skip hidden quotes to find what to display
                let display_idx = if !state.quotes[state.current_quote_index].is_hidden {
                    Some(state.current_quote_index)
                } else {
                    state.next_visible_index(state.current_quote_index)
                };

                if let Some(idx) = display_idx {
                    let main_text = state.quotes[idx].main_text.clone();
                    let sub_text  = state.quotes[idx].sub_text.clone();
                    render_quote_card(
                        ctx, ui, &main_text, &sub_text,
                        &state.text_style, state.title_bar_state.zoom_level,
                        shaper,
                    );
                } else {
                    // All hidden — soft notice card
                    ui.add_space(30.0);
                    let notice_w = 300.0_f32;
                    let (notice_rect, _) = ui.allocate_exact_size(
                        Vec2::new(notice_w, 56.0), Sense::hover(),
                    );
                    ui.painter().rect_filled(
                        notice_rect, Rounding::same(12.0),
                        Color32::from_rgba_unmultiplied(28, 28, 50, 180),
                    );
                    ui.painter().rect_stroke(
                        notice_rect, Rounding::same(12.0),
                        Stroke::new(1.0, NEON_SOLAR.gamma_multiply(0.45)),
                    );
                    ui.painter().text(
                        notice_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "All quotes are hidden",
                        FontId::proportional(15.0),
                        NEON_SOLAR.gamma_multiply(0.75),
                    );
                }

                ui.add_space(40.0);
            }
        });
    });
```

---

## STEP 4 — Update title bar quote counter to show hidden count

Inside `render_title_bar`, find:
```rust
if !state.quotes.is_empty() {
    ui.label(
        RichText::new(format!(
            "[ {}/{} ]",
            state.current_quote_index + 1,
            state.quotes.len()
        ))
        ...
    );
}
```

Replace with:
```rust
if !state.quotes.is_empty() {
    let visible = state.quotes.iter().filter(|q| !q.is_hidden).count();
    ui.label(
        RichText::new(format!(
            "[ {}/{} ]",
            state.current_quote_index + 1,
            state.quotes.len()
        ))
        .color(NEON_LIME.gamma_multiply(0.7))
        .size(10.5),
    );
    if visible < state.quotes.len() {
        ui.label(
            RichText::new(format!("({} hidden)", state.quotes.len() - visible))
                .color(NEON_SOLAR.gamma_multiply(0.6))
                .size(9.5),
        );
    }
}
```

---

## STEP 5 — Auto-rotation skips hidden quotes

In the `render()` method inside `AppRunner::render`, find:
```rust
if app_state.rotation_enabled
    && app_state.last_rotation.elapsed() >= app_state.rotation_interval
    && !app_state.quotes.is_empty()
{
    app_state.next_quote();
}
```
This already calls `next_quote()` which now uses `next_visible_index`, so **no change needed** — Step 1d handles it automatically.

---

## Visual result summary

```
┌─────────────── CONTROL PANEL LIST ITEM ────────────────┐
│  ▲  ┌─────────────────────────────────┬──┬──┐          │
│  ▼  │ 1. Quote main text here… ↳ sub  │◉ │✕ │          │
│     └─────────────────────────────────┴──┴──┘          │
└────────────────────────────────────────────────────────┘
     ^--- 14px arrow col   ^--- 1px gap between ◉ and ✕


┌─────────────── MAIN DISPLAY CARD ──────────────────────┐
│  ╔══════════════════════════════════════════╗           │
│  ║  [outer cyan glow — 3 expanding layers]  ║           │
│  ║  ┌──────────────────────────────────┐    ║           │
│  ║  │  ██  MAIN QUOTE TEXT  ██         │    ║           │
│  ║  │  ──── separator ────             │    ║           │
│  ║  │  supporting sub text             │    ║           │
│  ║  └──────────────────────────────────┘    ║           │
│  ╚══════════════════════════════════════════╝           │
└────────────────────────────────────────────────────────┘
  Semi-rounded (r=18), dark glass bg, corner tick marks
```
