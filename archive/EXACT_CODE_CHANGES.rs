// =============================================================================
// EXACT CODE CHANGES — copy-paste these replacements into your main.rs
// =============================================================================

// ─────────────────────────────────────────────────────────────────────────────
// REPLACEMENT BLOCK A
// Replace the ENTIRE opening of render_control_panel_contents (first ~10 lines)
// ─────────────────────────────────────────────────────────────────────────────

// OLD CODE (remove this):
// ─────────────────────────
//     ui.set_max_width(ui.available_width()); // Prevent horizontal overflow
//     egui::ScrollArea::vertical()
//         .auto_shrink([false, false])
//         .enable_scrolling(true)
//         .show(ui, |ui| {
//             ui.set_width(ui.available_width());

// NEW CODE (replace with this):
// ─────────────────────────────
    ui.set_max_width(ui.available_width());
    let panel_content_width = CONTROL_PANEL_WIDTH - 20.0; // stable fixed width for all inputs
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .enable_scrolling(true)
        .show(ui, |ui| {
            ui.set_width(panel_content_width);
            ui.set_max_width(panel_content_width);


// ─────────────────────────────────────────────────────────────────────────────
// REPLACEMENT BLOCK B
// In the ADD CUSTOM TEXT section — BOTH textarea horizontal blocks
// Replace BOTH occurrences of:
//     let text_width = (ui.available_width() - 80.0).max(50.0);
// With:
//     let text_width = (panel_content_width - 80.0).max(50.0);
// ─────────────────────────────────────────────────────────────────────────────


// ─────────────────────────────────────────────────────────────────────────────
// REPLACEMENT BLOCK C  
// In the TEXT LIST section inside render_section closure
// ─────────────────────────────────────────────────────────────────────────────

// OLD CODE (remove this):
// ─────────────────────────
//             let allocated_width = ui.available_width();
//
//             for (idx, quote) in state.quotes.iter().enumerate() {
//                 ...
//                 let total_available = allocated_width - 6.0;
//                 egui::Frame::none()
//                     ...
//                     .show(ui, |ui| {
//                         // Caculate internal width: total_available - margins(16) - stroke(2)
//                         let internal_width = (total_available - 18.0).max(10.0);
//                         ui.set_max_width(internal_width);
//                         ui.set_width(internal_width);

// NEW CODE (replace with this):
// ─────────────────────────────
            // Fixed width — captured ONCE so every box is identical
            let list_box_internal_width = (panel_content_width - 48.0).max(10.0);

            for (idx, quote) in state.quotes.iter().enumerate() {
                // ... (keep everything else the same) ...
                // Remove: let total_available = ...;
                // Change the Frame .show block to use:
                //     ui.set_max_width(list_box_internal_width);
                //     ui.set_width(list_box_internal_width);
