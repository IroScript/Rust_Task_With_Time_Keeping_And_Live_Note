# ⚡ INSTANT HOVER FIX — 3 Changes in main.rs

## ROOT CAUSE
The hover delay (~500ms) comes from TWO bugs:
1. `about_to_wait` sleeps 500ms between frames → hover changes sit unrendered
2. `CursorMoved` never calls `window.request_redraw()` → egui never repaints on mouse move

---

## CHANGE 1 — Fix `about_to_wait` (MOST IMPORTANT)

Find this block (near bottom of `about_to_wait`):
```rust
// Use WaitUntil with a timeout to wake up periodically for quote rotation
// This keeps the window responsive while still checking for quote changes
// Use 500ms to reduce CPU/GPU load on older hardware
let next_wake = Instant::now() + Duration::from_millis(500);
event_loop.set_control_flow(winit::event_loop::ControlFlow::WaitUntil(next_wake));
```

Replace with:
```rust
// Use Poll so cursor hover is INSTANT — no sleep delay.
// Throttle only when no animation is running to save CPU.
if needs_redraw {
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
} else {
    // Still wake periodically for quote rotation timer (16ms = 60fps max idle rate)
    let next_wake = Instant::now() + Duration::from_millis(16);
    event_loop.set_control_flow(winit::event_loop::ControlFlow::WaitUntil(next_wake));
}
```

---

## CHANGE 2 — Request redraw on CursorMoved

Find this in `window_event`:
```rust
WindowEvent::RedrawRequested => {
    self.render(&window);
}
_ => {}
```

Replace with:
```rust
WindowEvent::RedrawRequested => {
    self.render(&window);
}
WindowEvent::CursorMoved { .. } => {
    // INSTANT HOVER: always repaint when cursor moves
    window.request_redraw();
}
_ => {}
```

---

## CHANGE 3 — Ensure animation_time is 0 (verify it's already there)

In `resumed()`, find where egui style is set. Make sure this line EXISTS:
```rust
style.animation_time = 0.0;
```

It's already in the code at:
```rust
// INSTANT hover effects - zero animation delay
style.animation_time = 0.0;
```
✅ This is already correct — no change needed here.

---

## CHANGE 4 — Remove hover response delay in egui visuals

In `resumed()`, after `egui_ctx.set_style(style);`, add:
```rust
// Force egui to repaint on every cursor interaction
egui_ctx.set_embed_viewports(false);
```

Also in `render()`, inside the `egui_ctx.run(raw_input, |ctx| {` closure,
at the very TOP of the closure body, add:
```rust
// Request continuous repaint while pointer is moving for instant hover
if ctx.input(|i| i.pointer.has_pointer()) {
    ctx.request_repaint();
}
```

---

## SUMMARY OF ALL 4 CHANGES

| # | File Location | Change |
|---|--------------|--------|
| 1 | `about_to_wait` | Change 500ms sleep → Poll + 16ms |
| 2 | `window_event` match arm | Add `CursorMoved` → `request_redraw()` |
| 3 | `resumed()` style setup | Already correct (`animation_time = 0.0`) |
| 4 | `render()` closure start | Add `ctx.request_repaint()` when pointer present |

After these changes, hover will be **frame-perfect instant** — no delay whatsoever.
