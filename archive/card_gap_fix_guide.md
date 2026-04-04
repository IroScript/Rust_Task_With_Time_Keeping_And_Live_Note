# Card Side Gap Fix — Sample Code & AI Prompt Guide

## Problem Summary

The quote cards have large empty gaps on both left and right sides (visible in screenshot).
The cause is **two separate issues working together**:

1. `card_width` is calculated as `ui.available_width() * 0.88` — uses only 88% of space
2. `ui.vertical_centered(...)` wraps cards in a centered layout that adds extra horizontal margins

---

## ROOT CAUSE CODE (Current Broken Code)

### Issue 1 — `render_quote_card()` — card_width calculation
```rust
// ❌ BROKEN — 0.88 multiplier creates 12% dead space on each side
let card_width = if editing {
    ui.max_rect().width()
} else {
    (ui.available_width() * 0.88).min(720.0).max(280.0)  // <-- 0.88 is the problem
};
```

### Issue 2 — `render_main_content()` — vertical_centered wrapper
```rust
// ❌ BROKEN — vertical_centered adds centering margins that shrink available width
} else {
    // Centered layout when not editing
    ui.vertical_centered(|ui| {   // <-- this wrapper adds horizontal margins
        ui.add_space(80.0);
        // ... cards rendered inside here get reduced width
    });
}
```

---

## FIXED CODE

### Fix 1 — `render_quote_card()` — Remove the 0.88 multiplier
```rust
// ✅ FIXED — use full available width, no percentage shrink
let card_width = if editing {
    ui.max_rect().width()
} else {
    (ui.available_width()).min(720.0).max(280.0)  // Removed * 0.88
    // Note: keep the .min(720.0) cap if you want a max width, 
    // or remove it too for truly full-width cards:
    // ui.available_width().max(280.0)
};
```

### Fix 2 — `render_main_content()` — Replace `vertical_centered` with `vertical`
```rust
// ✅ FIXED — use vertical() instead of vertical_centered() to eliminate side margins
} else {
    // Full width layout — no centering gap
    ui.vertical(|ui| {
        ui.add_space(80.0);

        // 1. Preview card
        if !state.main_text_input.is_empty() && state.editing_quote_index.is_none() {
            render_quote_card(
                ctx, ui, state, None, shaper,
                ui.id().with("preview_quote_card"),
            );
            ui.add_space(30.0);
        }

        // 2. All visible quotes
        let mut visible_count = 0;
        for idx in 0..state.quotes.len() {
            let is_hidden = state.quotes[idx].is_hidden;
            if idx == state.current_quote_index || !is_hidden {
                let card_id = egui::Id::new("quote_card").with(idx);
                let is_editing = state.editing_quote_index == Some(idx);

                render_quote_card(ctx, ui, state, Some(idx), shaper, card_id);

                if is_editing && ui.input(|i| i.key_pressed(egui::Key::Enter) && !i.modifiers.shift) {
                    state.save_current_input();
                }
                ui.add_space(30.0);
                visible_count += 1;
            }
        }

        if visible_count == 0 && state.main_text_input.is_empty() {
            ui.label(
                RichText::new("No visible quotes. Add one or unhide from the control panel!")
                    .color(Color32::GRAY)
                    .size(18.0),
            );
        }

        ui.add_space(60.0);
    });
}
```

### Fix 3 (Optional) — Also remove inner_margin from CentralPanel if any gap remains
```rust
// ✅ Make sure CentralPanel has zero margin
egui::CentralPanel::default()
    .frame(Frame::none().fill(Color32::TRANSPARENT).inner_margin(0.0))  // inner_margin(0.0) is key
    .show(ctx, |ui| {
        // ...
    });
```

### Fix 4 (Optional) — Zero out item_spacing.x globally in the scroll area
```rust
egui::ScrollArea::vertical()
    .auto_shrink([false, false])
    .show(ui, |ui| {
        ui.spacing_mut().item_spacing.x = 0.0;  // Eliminate any horizontal item gaps
        // ... rest of content
    });
```

---

## COMPLETE BEFORE/AFTER DIFF

```diff
// In render_quote_card():
- let card_width = if editing {
-     ui.max_rect().width()
- } else {
-     (ui.available_width() * 0.88).min(720.0).max(280.0)
- };
+ let card_width = if editing {
+     ui.max_rect().width()
+ } else {
+     ui.available_width().max(280.0)
+ };

// In render_main_content() — the else branch:
- ui.vertical_centered(|ui| {
+ ui.vertical(|ui| {
      ui.add_space(80.0);
      // ... (rest of content is identical, no other changes needed)
```

---

## AI CODING TOOL PROMPT

Copy and paste this prompt directly to Cursor, Copilot, or any AI coding tool:

---

```
I have a Rust egui application with quote cards that display large empty gaps on 
both the left and right sides. I need you to fix two specific issues:

ISSUE 1: In the function `render_quote_card()`, find this line:
    (ui.available_width() * 0.88).min(720.0).max(280.0)
Change it to:
    ui.available_width().max(280.0)
Remove the `* 0.88` multiplier entirely. This 0.88 factor was shrinking 
the card to 88% of available width, leaving 12% empty on the sides.

ISSUE 2: In the function `render_main_content()`, inside the ScrollArea, 
there is an else branch (when NOT editing) that wraps cards in:
    ui.vertical_centered(|ui| { ... })
Change `vertical_centered` to `vertical`:
    ui.vertical(|ui| { ... })
The content inside the closure stays EXACTLY the same — only the wrapper 
function name changes. `vertical_centered` adds centering margins that 
create horizontal gaps; `vertical` uses full available width.

ISSUE 3 (if gaps still remain): Ensure CentralPanel has inner_margin(0.0):
    egui::CentralPanel::default()
        .frame(Frame::none().fill(Color32::TRANSPARENT).inner_margin(0.0))

Do NOT change any other logic — hover effects, edit mode, rotation, Bengali 
text rendering, and double-click handlers must remain exactly the same.
Only change what is listed above.
```

---

## VERIFICATION CHECKLIST

After applying the fix, verify:

- [ ] Cards stretch edge-to-edge in the central panel (no side gaps)
- [ ] Hover glow effect still appears on card borders
- [ ] Cards still have the correct dark glass background
- [ ] Edit mode (double-click) still enters correctly
- [ ] Bengali text still renders properly
- [ ] Corner accent ticks still appear
- [ ] Control panel on the right side still works
- [ ] The card background (glow layers + fill + border) resizes with the wider card

---

## WHY vertical_centered CAUSES GAPS

`ui.vertical_centered(|ui| { ... })` in egui works by:
1. Calculating the available width
2. Centering child widgets by adding `(available_width - child_width) / 2` as left margin
3. This means even if a child widget requests full width, the parent has already 
   consumed some horizontal space for centering math

`ui.vertical(|ui| { ... })` simply stacks children top-to-bottom using the 
full available width — which is what we want here since the cards should 
fill the entire central panel area.
