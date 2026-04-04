# ================================================================
# HOW TO INTEGRATE — READ THIS FIRST
# ================================================================

## What's in this delivery

| File                  | Purpose                                             |
|-----------------------|-----------------------------------------------------|
| `native_drag.rs`      | WM_NCHITTEST hook → OS-native drag, zero latency    |
| `cpu_render.rs`       | softbuffer + scanline rasteriser → CPU-only render  |
| `app_runner_new.rs`   | Drop-in AppRunner with all changes already merged   |
| `CHANGES_TO_MAIN_RS.md` | Step-by-step diff reference                       |

---

## Step 1 — Copy the 3 Rust files into your `src/` folder

```
src/
  main.rs          ← your existing file
  native_drag.rs   ← new (copy from delivery)
  cpu_render.rs    ← new (copy from delivery)
  app_runner_new.rs ← reference only, changes merged into main.rs below
```

---

## Step 2 — Cargo.toml additions

Add under `[dependencies]`:
```toml
softbuffer = "0.4"
```

Remove or leave `wgpu` / `egui_wgpu` — they are only needed by `quantum_logo.exe`
(the background binary), not by `main.rs` any more.

---

## Step 3 — main.rs: 6 surgical edits

### 3-A  Add module declarations (near top, after existing `use` blocks)
```rust
mod native_drag;      // WM_NCHITTEST zero-latency drag
mod cpu_render;       // softbuffer CPU renderer
use cpu_render::CpuRenderState;
use std::sync::Arc;
```

### 3-B  Replace `AppRunner` struct definition
Replace:
```rust
struct AppRunner {
    window: Option<&'static Window>,
    render_state: Option<WgpuRenderState<'static>>,
    // ...
    should_close: bool,
}
```
With:
```rust
struct AppRunner {
    window:               Option<Arc<Window>>,
    render_state:         Option<CpuRenderState>,
    app_state:            Option<AppState>,
    egui_ctx:             Option<Context>,
    egui_state:           Option<egui_winit::State>,
    font_system:          Option<cosmic_text::FontSystem>,
    swash_cache:          Option<cosmic_text::SwashCache>,
    shaped_text_textures: HashMap<u64, egui::TextureHandle>,
    should_close:         bool,
    cursor_pos:           Option<winit::dpi::PhysicalPosition<f64>>, // ← NEW
}
```

### 3-C  In `resumed()` — switch to Arc + install native drag

Replace:
```rust
let window = Box::leak(Box::new(window));
```
With:
```rust
let window: Arc<Window> = Arc::new(window);
```

Inside the existing `#[cfg(windows)]` block, right after `set_window_topmost(hwnd)`:
```rust
crate::native_drag::install(hwnd);   // ← zero-latency drag
```

Replace:
```rust
match pollster::block_on(WgpuRenderState::new(window)) {
```
With:
```rust
match CpuRenderState::new(window.clone()) {
```

Replace assignment:
```rust
self.render_state = Some(render_state);
// ...
self.window = Some(window);
```
With:
```rust
self.render_state = Some(render_state);
// ... (egui_ctx, egui_state, etc unchanged)
self.window = Some(window);  // window is now Arc<Window>
```

### 3-D  In `window_event()` — immediate drag on mouse press

Add these lines at the very TOP of the `window_event` function body,
BEFORE the existing `egui_state.on_window_event(...)` call:

```rust
// Track cursor for immediate drag
if let WindowEvent::CursorMoved { position, .. } = &event {
    self.cursor_pos = Some(*position);
}

// ★ DRAG FIX: call drag_window() immediately on press — before egui sees the event
if let WindowEvent::MouseInput {
    state: winit::event::ElementState::Pressed,
    button: winit::event::MouseButton::Left, ..
} = &event {
    if let (Some(window), Some(pos)) = (self.window.as_ref(), self.cursor_pos) {
        let scale = window.scale_factor();
        let ly    = pos.y / scale;
        let lx    = pos.x / scale;
        let lw    = window.inner_size().width as f64 / scale;
        // Title-bar drag strip, excluding right buttons (≈450 logical px)
        if ly >= 0.0 && ly < TITLE_BAR_HEIGHT as f64 && lx >= 8.0 && lx < lw - 450.0 {
            let _ = window.drag_window();
        }
    }
}
```

### 3-E  In `Resized` branch
```rust
WindowEvent::Resized(size) => {
    if let Some(rs) = self.render_state.as_mut() {
        rs.resize(size.width, size.height);   // CpuRenderState::resize
    }
}
```

### 3-F  Replace the bottom of `render()` — swap wgpu submit for CPU blit

Delete everything from `let frame = match render_state.surface.get_current_texture()...`
to `frame.present();`  and replace with:

```rust
// ── CPU render — pure software, no GPU ────────────────────────────
let bg = app_state.get_background_color();
render_state.render(&paint_jobs, &full_output.textures_delta, scale, bg);
```

---

## Why this makes dragging as fast as Notepad

```
BEFORE (egui path):
  Mouse press → winit event queue → about_to_wait wakeup
  → RedrawRequested → egui frame → drag_window() posted
  Latency: 1-3 frames  ≈ 16-50 ms

AFTER (WM_NCHITTEST path):
  Windows sends WM_NCHITTEST BEFORE mouse button events.
  Our proc returns HTCAPTION → OS starts native move loop immediately.
  egui never sees the click.
  Latency: 0 ms  (same kernel path as Notepad, Chrome, everything)
```

## Why CPU rendering is clean

```
BEFORE:  main.rs → wgpu → GPU → display
AFTER:   main.rs → cpu_render → softbuffer pixel buffer → display

quantum_logo.exe (background) → still uses wgpu → GPU (unchanged)
```

The two renderers never touch the same hardware resource.

---

## Tuning the drag zone

If buttons are accidentally draggable, decrease the right dead-zone:
```rust
// At runtime (or at startup in main()):
crate::native_drag::RIGHT_BUTTONS_PX.store(500, std::sync::atomic::Ordering::Relaxed);
```

If the left edge is too wide, adjust:
```rust
crate::native_drag::LEFT_DEAD_PX.store(4, std::sync::atomic::Ordering::Relaxed);
```
