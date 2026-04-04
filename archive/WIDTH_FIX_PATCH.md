# Fix: Inconsistent Text Field Box Widths in Control Panel

## The Problem
All text input fields and list item boxes in the right panel should be the same width, 
but they grow larger with each one. This is caused by `ui.available_width()` returning 
different values inside nested layouts.

---

## CHANGE 1 — Fix `render_control_panel_contents` function start

**FIND this at the top of the function body (inside the ScrollArea):**
```rust
ui.set_max_width(ui.available_width()); // Prevent horizontal overflow
egui::ScrollArea::vertical()
    .auto_shrink([false, false])
    .enable_scrolling(true)
    .show(ui, |ui| {
        ui.set_width(ui.available_width());
```

**REPLACE WITH:**
```rust
ui.set_max_width(ui.available_width()); // Prevent horizontal overflow
// Capture stable panel content width ONCE before any layout operations
let panel_content_width = CONTROL_PANEL_WIDTH - 20.0; // subtract left+right margins
egui::ScrollArea::vertical()
    .auto_shrink([false, false])
    .enable_scrolling(true)
    .show(ui, |ui| {
        ui.set_width(panel_content_width);
        ui.set_max_width(panel_content_width);
```

---

## CHANGE 2 — Fix main text input width

**FIND (inside the "ADD CUSTOM TEXT" section, first horizontal block):**
```rust
ui.horizontal(|ui| {
    // Textarea on the left
    let text_width = (ui.available_width() - 80.0).max(50.0);
    let mut text_response = None;
```

**REPLACE WITH:**
```rust
ui.horizontal(|ui| {
    // Textarea on the left — use fixed width so all inputs are same size
    let text_width = (panel_content_width - 80.0).max(50.0);
    let mut text_response = None;
```

---

## CHANGE 3 — Fix supporting/sub text input width

**FIND (the second textarea block, a few lines below):**
```rust
ui.horizontal(|ui| {
    let text_width = (ui.available_width() - 80.0).max(50.0);
    let mut sub_response = None;
```

**REPLACE WITH:**
```rust
ui.horizontal(|ui| {
    let text_width = (panel_content_width - 80.0).max(50.0);
    let mut sub_response = None;
```

---

## CHANGE 4 — Fix text list item boxes

**FIND (inside the "TEXT LIST" section's `render_section` call):**
```rust
let allocated_width = ui.available_width();

for (idx, quote) in state.quotes.iter().enumerate() {
    ...
    let total_available = allocated_width - 6.0;
    egui::Frame::none()
        ...
        .show(ui, |ui| {
            // Caculate internal width: total_available - margins(16) - stroke(2)
            let internal_width = (total_available - 18.0).max(10.0);
            ui.set_max_width(internal_width);
            ui.set_width(internal_width); // Force consistent width for all boxes
```

**REPLACE WITH:**
```rust
// Use fixed panel content width so ALL list boxes are the same size
let fixed_box_width = panel_content_width - 6.0;

for (idx, quote) in state.quotes.iter().enumerate() {
    ...
    let total_available = fixed_box_width;
    egui::Frame::none()
        ...
        .show(ui, |ui| {
            // Fixed internal width: panel width - margins(16) - stroke(2) - section padding(24)
            let internal_width = (panel_content_width - 48.0).max(10.0);
            ui.set_max_width(internal_width);
            ui.set_width(internal_width); // Force consistent width for all boxes
```

---

## CHANGE 5 — Pass `panel_content_width` into `render_section` closures

Since `panel_content_width` is defined in the outer function but used inside 
`render_section` closures, Rust's borrow checker allows this via capture.
The closures `|ui| { ... }` already capture from the enclosing scope, so 
`panel_content_width` will be available automatically — no extra changes needed.

---

## SUMMARY OF ALL OCCURRENCES TO CHANGE

| # | Search for | Replace with |
|---|-----------|-------------|
| 1 | `ui.set_width(ui.available_width());` (in ScrollArea) | `ui.set_width(panel_content_width); ui.set_max_width(panel_content_width);` |
| 2 | `let text_width = (ui.available_width() - 80.0).max(50.0);` (main input) | `let text_width = (panel_content_width - 80.0).max(50.0);` |
| 3 | `let text_width = (ui.available_width() - 80.0).max(50.0);` (sub input) | `let text_width = (panel_content_width - 80.0).max(50.0);` |
| 4 | `let allocated_width = ui.available_width();` | `let fixed_box_width = panel_content_width - 6.0;` |
| 5 | `let total_available = allocated_width - 6.0;` | `let total_available = fixed_box_width;` |
| 6 | `let internal_width = (total_available - 18.0).max(10.0);` | `let internal_width = (panel_content_width - 48.0).max(10.0);` |

---

## WHY THIS WORKS

`ui.available_width()` inside a `ui.horizontal()` or nested `Frame` returns the 
**remaining** width after previously placed widgets — this shrinks each call.

By capturing `panel_content_width = CONTROL_PANEL_WIDTH - 20.0` **once** before 
any layout operations, we get a stable, consistent value that all boxes use equally.
