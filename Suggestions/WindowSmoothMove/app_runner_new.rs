// app_runner_new.rs
// ======================================================================
// Drop-in replacement for the AppRunner struct and its impl blocks.
//
// Changes vs original:
//   1. `render_state` → CpuRenderState  (softbuffer, CPU only)
//   2. `cursor_pos`   → tracks mouse position for immediate drag
//   3. `window`       → Arc<Window>  (shared ownership, no 'static leak)
//   4. `resumed()`    → installs WM_NCHITTEST handler for native drag
//   5. `window_event` → calls drag_window() on press — BEFORE egui frame
//   6. `render()`     → uses cpu_render pipeline; wgpu removed from main
// ======================================================================

use std::{collections::HashMap, sync::Arc, time::{Duration, Instant}};
use winit::{
    application::ApplicationHandler,
    dpi::{LogicalSize, PhysicalPosition},
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::Window,
};
use egui::{Context, Pos2, Rect};

use crate::{
    cpu_render::CpuRenderState,
    AppAnimation, AppState, TITLE_BAR_HEIGHT,
    transform_content_shapes, transform_raw_input_for_rotation_scale,
    render_title_bar, render_main_content, render_theme_modal, render_profile_modal,
    render_floating_buttons, TitleBarAction, DEFAULT_WINDOW_SIZE, MIN_WINDOW_SIZE,
};

#[cfg(windows)]
use windows::Win32::Foundation::HWND;
#[cfg(windows)]
use windows::Win32::UI::WindowsAndMessaging::{
    GetWindowLongW, SetLayeredWindowAttributes, SetPropW, SetWindowPos,
    GWL_EXSTYLE, HWND_TOPMOST, LWA_ALPHA, SWP_NOMOVE, SWP_NOSIZE, SWP_SHOWWINDOW, WS_EX_LAYERED,
};

// ─── helper (unchanged) ────────────────────────────────────────────────
fn log_to_file(msg: &str) {
    use std::io::Write;
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open("debug.log") {
        let _ = writeln!(f, "{}", msg);
    }
}

#[cfg(windows)]
fn set_window_topmost(hwnd: HWND) {
    unsafe {
        let _ = SetWindowPos(hwnd, HWND_TOPMOST, 0, 0, 0, 0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_SHOWWINDOW);
    }
}

#[cfg(windows)]
fn get_global_cursor() -> Option<(i32, i32)> {
    use windows::Win32::Foundation::POINT;
    use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;
    let mut pt = POINT::default();
    if unsafe { GetCursorPos(&mut pt) }.is_ok() { Some((pt.x, pt.y)) } else { None }
}
#[cfg(not(windows))]
fn get_global_cursor() -> Option<(i32, i32)> { None }

// ─── AppRunner ─────────────────────────────────────────────────────────
pub struct AppRunner {
    window:               Option<Arc<Window>>,       // Arc — no 'static leak
    render_state:         Option<CpuRenderState>,    // CPU renderer (softbuffer)
    app_state:            Option<AppState>,
    egui_ctx:             Option<Context>,
    egui_state:           Option<egui_winit::State>,
    font_system:          Option<cosmic_text::FontSystem>,
    swash_cache:          Option<cosmic_text::SwashCache>,
    shaped_text_textures: HashMap<u64, egui::TextureHandle>,
    should_close:         bool,
    cursor_pos:           Option<PhysicalPosition<f64>>, // ← NEW: for immediate drag
}

impl Default for AppRunner {
    fn default() -> Self {
        Self {
            window:               None,
            render_state:         None,
            app_state:            None,
            egui_ctx:             None,
            egui_state:           None,
            font_system:          Some(cosmic_text::FontSystem::new()),
            swash_cache:          Some(cosmic_text::SwashCache::new()),
            shaped_text_textures: HashMap::new(),
            should_close:         false,
            cursor_pos:           None,
        }
    }
}

// ─── ApplicationHandler ────────────────────────────────────────────────
impl ApplicationHandler for AppRunner {

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() { return; }
        log_to_file("resumed() — creating window");

        let raw_window = match event_loop.create_window(
            Window::default_attributes()
                .with_title("Daily Motivation")
                .with_inner_size(LogicalSize::new(DEFAULT_WINDOW_SIZE.0, DEFAULT_WINDOW_SIZE.1))
                .with_min_inner_size(LogicalSize::new(MIN_WINDOW_SIZE.0,  MIN_WINDOW_SIZE.1))
                .with_decorations(false)
                .with_resizable(true)
                .with_transparent(true)
                .with_visible(false),
        ) {
            Ok(w) => w,
            Err(e) => { eprintln!("Window creation failed: {e}"); event_loop.exit(); return; }
        };

        let window = Arc::new(raw_window);

        // ── Windows-specific: topmost + NATIVE DRAG ─────────────────
        #[cfg(windows)]
        {
            use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
            if let Ok(handle) = window.window_handle() {
                if let RawWindowHandle::Win32(w32) = handle.as_raw() {
                    let hwnd = HWND(w32.hwnd.get() as *mut _);
                    set_window_topmost(hwnd);

                    // ── KEY FIX: zero-latency drag via WM_NCHITTEST ──────
                    // This makes dragging IDENTICAL to Notepad / Chrome.
                    // The OS intercepts the mouse press and moves the window
                    // directly — egui never sees the event, no frame latency.
                    crate::native_drag::install(hwnd);
                }
            }
        }

        // ── CPU render state (softbuffer — no GPU) ───────────────────
        match CpuRenderState::new(window.clone()) {
            Ok(render_state) => {
                let app_state  = AppState::default();
                let egui_ctx   = Context::default();

                // Style (unchanged from original)
                let mut style  = egui::Style::default();
                style.visuals   = egui::Visuals::dark();
                style.visuals.window_fill = egui::Color32::TRANSPARENT;
                style.visuals.panel_fill  = egui::Color32::TRANSPARENT;
                {
                    use egui::Color32;
                    use crate::NEON_CYAN;
                    let mut v = style.visuals.clone();
                    v.widgets.hovered.bg_fill   = Color32::from_rgb(80,  80, 90);
                    v.widgets.hovered.bg_stroke  = egui::Stroke::new(1.0, Color32::WHITE.gamma_multiply(0.5));
                    v.widgets.active.bg_fill     = Color32::from_rgb(100, 100, 110);
                    v.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, Color32::from_rgba_unmultiplied(190, 230, 255, 255));
                    v.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, Color32::WHITE);
                    v.widgets.active.fg_stroke   = egui::Stroke::new(1.0, NEON_CYAN);
                    v.widgets.hovered.fg_stroke  = egui::Stroke::new(1.0, NEON_CYAN);
                    style.visuals = v;
                }
                egui_ctx.set_style(style);

                let egui_state = egui_winit::State::new(
                    egui_ctx.clone(), egui::ViewportId::ROOT,
                    window.as_ref(), None, None, None,
                );

                crate::setup_fonts(&egui_ctx);

                self.render_state = Some(render_state);
                self.app_state    = Some(app_state);
                self.egui_ctx     = Some(egui_ctx);
                self.egui_state   = Some(egui_state);
                self.window       = Some(window.clone());

                window.set_visible(true);
                log_to_file("CPU render state ready");
            }
            Err(e) => {
                eprintln!("CPU render init failed: {e}");
                log_to_file(&format!("CPU render init failed: {e}"));
                event_loop.exit();
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _wid: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let Some(window) = self.window.clone() else { return };

        // ── 1. Track cursor position for immediate drag detection ────
        if let WindowEvent::CursorMoved { position, .. } = &event {
            self.cursor_pos = Some(*position);
        }

        // ── 2. IMMEDIATE drag — fires BEFORE egui sees the event ────
        //    On Windows this is redundant (WM_NCHITTEST already handles it)
        //    but it covers non-Windows platforms and the panel-drag area.
        if let WindowEvent::MouseInput {
            state: winit::event::ElementState::Pressed,
            button: winit::event::MouseButton::Left, ..
        } = &event {
            if let Some(pos) = self.cursor_pos {
                let scale  = window.scale_factor();
                let log_y  = pos.y / scale;
                let log_x  = pos.x / scale;
                let log_w  = window.inner_size().width as f64 / scale;

                // Title-bar drag strip (exclude right buttons ~450 logical px)
                if log_y >= 0.0
                    && log_y <  TITLE_BAR_HEIGHT as f64
                    && log_x >= 8.0
                    && log_x <  log_w - 450.0
                {
                    let _ = window.drag_window();  // immediate, before egui frame
                }
            }
        }

        // ── 3. Forward to egui ──────────────────────────────────────
        if let Some(egui_state) = self.egui_state.as_mut() {
            let resp = egui_state.on_window_event(window.as_ref(), &event);
            if resp.repaint { window.request_redraw(); }
        }

        match &event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::Resized(size) => {
                if let Some(rs) = self.render_state.as_mut() {
                    rs.resize(size.width, size.height);
                }
            }

            WindowEvent::RedrawRequested => self.render(window.as_ref()),

            _ => {}
        }

        // Update interaction time + request repaint on input
        if let Some(app_state) = self.app_state.as_mut() {
            match &event {
                WindowEvent::CursorMoved { .. }
                | WindowEvent::MouseInput { .. }
                | WindowEvent::KeyboardInput { .. } => {
                    app_state.last_interaction = Instant::now();

                    // Stop animations on Space
                    if let WindowEvent::KeyboardInput { event: ke, .. } = &event {
                        if ke.state == winit::event::ElementState::Pressed {
                            if let winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Space) = ke.physical_key {
                                app_state.active_animation = AppAnimation::None;
                                #[cfg(windows)]
                                if let Ok(h) = window.window_handle() {
                                    if let winit::raw_window_handle::RawWindowHandle::Win32(w32) = h.as_raw() {
                                        let hwnd = HWND(w32.hwnd.get() as *mut _);
                                        unsafe { let _ = SetLayeredWindowAttributes(hwnd, None, 255, LWA_ALPHA); }
                                    }
                                }
                            }
                        }
                    }

                    window.request_redraw();
                }
                _ => {}
            }
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if self.should_close { event_loop.exit(); return; }

        if let Some(window) = &self.window {
            let needs_redraw = self.app_state.as_ref().map_or(false, |s| {
                s.active_animation != AppAnimation::None
                || (s.current_rotation_angle - s.target_rotation_angle).abs() > 0.001
            });
            if needs_redraw { window.request_redraw(); }
        }

        // Wake every 500 ms to handle quote rotation timer
        let next = Instant::now() + Duration::from_millis(500);
        event_loop.set_control_flow(winit::event_loop::ControlFlow::WaitUntil(next));
    }
}

// ─── render() ──────────────────────────────────────────────────────────
impl AppRunner {
    fn render(&mut self, window: &Window) {
        if self.app_state.is_none()
            || self.egui_ctx.is_none()
            || self.egui_state.is_none()
            || self.render_state.is_none()
        { return; }

        let mut font_system = self.font_system.take();
        let mut swash_cache = self.swash_cache.take();
        let mut tex_cache   = std::mem::take(&mut self.shaped_text_textures);

        let (app_state, egui_ctx, egui_state, render_state) = match (
            self.app_state.as_mut(),
            self.egui_ctx.as_mut(),
            self.egui_state.as_mut(),
            self.render_state.as_mut(),
        ) {
            (Some(a), Some(b), Some(c), Some(d)) => (a, b, c, d),
            _ => {
                self.font_system = font_system;
                self.swash_cache = swash_cache;
                self.shaped_text_textures = tex_cache;
                return;
            }
        };

        let scale     = window.scale_factor() as f32;
        let content_w = window.inner_size().width  as f32 / scale;
        let content_h = window.inner_size().height as f32 / scale;
        let content_rect = Rect::from_min_max(
            Pos2::new(0.0, TITLE_BAR_HEIGHT),
            Pos2::new(content_w, content_h),
        );

        let mut raw_input = egui_state.take_egui_input(window);
        transform_raw_input_for_rotation_scale(
            &mut raw_input, content_rect,
            app_state.current_rotation_angle,
            app_state.current_scale,
        );

        // ── egui frame (ALL original logic preserved) ──────────────
        let full_output = egui_ctx.run(raw_input, |ctx| {
            if ctx.is_using_pointer()
                || ctx.input(|i| i.pointer.any_down() || !i.events.is_empty())
            {
                app_state.last_interaction = Instant::now();
            }

            // ── resize border handling (unchanged) ─────────────────
            let mut is_resizing = false;
            if let Some((dir, sx, sy, wx, wy, sw, sh)) = app_state.manual_resize_start {
                is_resizing = true;
                if ctx.input(|i| i.pointer.primary_down()) {
                    if let Some((cx, cy)) = get_global_cursor() {
                        let (dx, dy) = (cx - sx, cy - sy);
                        let mut nw = sw as i32; let mut nh = sh as i32;
                        let mut nx = wx;        let mut ny = wy;
                        use winit::window::ResizeDirection::*;
                        match dir {
                            East  => nw += dx,
                            West  => { nw -= dx; nx += dx; }
                            South => nh += dy,
                            North => { nh -= dy; ny += dy; }
                            SouthEast => { nw += dx; nh += dy; }
                            SouthWest => { nw -= dx; nx += dx; nh += dy; }
                            NorthEast => { nw += dx; nh -= dy; ny += dy; }
                            NorthWest => { nw -= dx; nx += dx; nh -= dy; ny += dy; }
                        }
                        window.set_outer_position(winit::dpi::PhysicalPosition::new(nx, ny));
                        let _ = window.request_inner_size(winit::dpi::PhysicalSize::new(nw.max(0) as u32, nh.max(0) as u32));
                    }
                } else {
                    app_state.manual_resize_start = None;
                }
            }
            let border = 8.0;
            let screen_rect = ctx.screen_rect();
            if !is_resizing {
                if let Some(pos) = ctx.input(|i| i.pointer.hover_pos()) {
                    let (l, r, t, b) = (pos.x < border, pos.x > screen_rect.max.x - border,
                                        pos.y < border, pos.y > screen_rect.max.y - border);
                    if l || r || t || b {
                        if t&&l { ctx.set_cursor_icon(egui::CursorIcon::ResizeNwSe); }
                        else if t&&r { ctx.set_cursor_icon(egui::CursorIcon::ResizeNeSw); }
                        else if b&&l { ctx.set_cursor_icon(egui::CursorIcon::ResizeNeSw); }
                        else if b&&r { ctx.set_cursor_icon(egui::CursorIcon::ResizeNwSe); }
                        else if t||b { ctx.set_cursor_icon(egui::CursorIcon::ResizeVertical); }
                        else         { ctx.set_cursor_icon(egui::CursorIcon::ResizeHorizontal); }
                        if ctx.input(|i| i.pointer.primary_pressed()) {
                            use winit::window::ResizeDirection::*;
                            let dir = if t&&l { NorthWest } else if t&&r { NorthEast }
                                      else if b&&l { SouthWest } else if b&&r { SouthEast }
                                      else if t { North } else if b { South }
                                      else if l { West  } else { East };
                            if let (Some((cx,cy)), Ok(wp)) = (get_global_cursor(), window.outer_position()) {
                                let sz = window.inner_size();
                                app_state.manual_resize_start = Some((dir, cx, cy, wp.x, wp.y, sz.width, sz.height));
                            } else {
                                let _ = window.drag_resize_window(dir);
                            }
                        }
                    }
                }
            }

            // ── title bar + actions (unchanged) ────────────────────
            let mut actions = render_title_bar(ctx, app_state, window);

            for action in &actions {
                match action {
                    TitleBarAction::ThemeClicked    => app_state.theme_modal_open   = true,
                    TitleBarAction::ProfileClicked  => app_state.profile_modal_open = true,
                    TitleBarAction::ZoomIn   => app_state.title_bar_state.zoom_level = (app_state.title_bar_state.zoom_level + 0.1).min(2.0),
                    TitleBarAction::ZoomOut  => app_state.title_bar_state.zoom_level = (app_state.title_bar_state.zoom_level - 0.1).max(0.5),
                    TitleBarAction::TogglePanel  => app_state.title_bar_state.control_panel_visible = !app_state.title_bar_state.control_panel_visible,
                    TitleBarAction::MinimizeClicked  => window.set_minimized(true),
                    TitleBarAction::MaximizeClicked  => window.set_maximized(!window.is_maximized()),
                    TitleBarAction::CloseClicked     => self.should_close = true,
                    TitleBarAction::HideHeader       => app_state.title_bar_state.header_visible = false,
                    TitleBarAction::ShowHeader       => app_state.title_bar_state.header_visible = true,
                    TitleBarAction::ToggleSingleQuote => {
                        app_state.single_quote_mode = !app_state.single_quote_mode;
                        app_state.save();
                    }
                    TitleBarAction::ExportClicked => {
                        use std::io::Write;
                        if let Ok(json) = serde_json::to_string_pretty(&app_state.quotes) {
                            if let Ok(mut f) = std::fs::OpenOptions::new().create(true).write(true).truncate(true).open("quotes_export.json") {
                                let _ = f.write_all(json.as_bytes());
                            }
                        }
                    }
                    TitleBarAction::PlayBounce  => { if app_state.active_animation == AppAnimation::None { if let Ok(p) = window.outer_position() { app_state.base_pos = Some((p.x, p.y)); } } app_state.active_animation = if app_state.active_animation == AppAnimation::Bounce { AppAnimation::None } else { AppAnimation::Bounce }; }
                    TitleBarAction::PlayShake   => { if app_state.active_animation == AppAnimation::None { if let Ok(p) = window.outer_position() { app_state.base_pos = Some((p.x, p.y)); } } app_state.active_animation = if app_state.active_animation == AppAnimation::Shake  { AppAnimation::None } else { AppAnimation::Shake  }; }
                    TitleBarAction::PlayDance   => { if app_state.active_animation == AppAnimation::None { if let Ok(p) = window.outer_position() { app_state.base_pos = Some((p.x, p.y)); } } app_state.active_animation = if app_state.active_animation == AppAnimation::Dance  { AppAnimation::None } else { AppAnimation::Dance  }; }
                    TitleBarAction::PlayRotate  => { app_state.rotation = app_state.rotation.wrapping_add(1); app_state.target_rotation_angle = app_state.rotation as f32 * std::f32::consts::FRAC_PI_2; }
                    TitleBarAction::PlayFly     => { if app_state.active_animation == AppAnimation::None { if let Ok(p) = window.outer_position() { app_state.base_pos = Some((p.x, p.y)); } } app_state.active_animation = if app_state.active_animation == AppAnimation::Fly     { AppAnimation::None } else { AppAnimation::Fly     }; }
                    TitleBarAction::PlayDissolve => {
                        if app_state.active_animation == AppAnimation::None { if let Ok(p) = window.outer_position() { app_state.base_pos = Some((p.x, p.y)); } }
                        app_state.active_animation = if app_state.active_animation == AppAnimation::Dissolve { AppAnimation::None } else { AppAnimation::Dissolve };
                        if app_state.active_animation == AppAnimation::None {
                            #[cfg(windows)]
                            if let Ok(h) = window.window_handle() { if let winit::raw_window_handle::RawWindowHandle::Win32(w32) = h.as_raw() { let hwnd = HWND(w32.hwnd.get() as *mut _); unsafe { let _ = SetLayeredWindowAttributes(hwnd, None, 255, LWA_ALPHA); } } }
                        }
                    }
                    TitleBarAction::ToggleBg => {
                        app_state.is_3d_bg_active = !app_state.is_3d_bg_active;
                        if app_state.is_3d_bg_active {
                            if app_state.bg_process.is_none() {
                                let sz  = window.inner_size();
                                let pos = window.outer_position().unwrap_or_default();
                                #[cfg(windows)]
                                {
                                    use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
                                    let mut main_hwnd: isize = 0;
                                    if let Ok(h) = window.window_handle() { if let RawWindowHandle::Win32(w32) = h.as_raw() { main_hwnd = w32.hwnd.get() as isize; } }
                                    let exe_paths = ["quantum_logo.exe", "background/target/release/quantum_logo.exe"];
                                    for path in exe_paths {
                                        if std::path::Path::new(path).exists() {
                                            if let Ok(child) = std::process::Command::new(path)
                                                .args([&sz.width.to_string(), &sz.height.to_string(), &pos.x.to_string(), &pos.y.to_string(), &main_hwnd.to_string()])
                                                .spawn() { app_state.bg_process = Some(child); break; }
                                        }
                                    }
                                }
                            }
                        } else if let Some(mut child) = app_state.bg_process.take() {
                            let _ = child.kill(); let _ = child.wait();
                        }
                    }
                    _ => {}
                }
            }

            // ── animation engine (unchanged) ───────────────────────
            if app_state.active_animation != AppAnimation::None {
                if let (Ok(pos), Some(mon)) = (window.outer_position(), window.current_monitor()) {
                    let sz  = window.outer_size();
                    let mz  = mon.size();
                    app_state.anim_progress += 0.016;
                    if app_state.base_pos.is_none() { app_state.base_pos = Some((pos.x, pos.y)); }
                    let (bx, by) = app_state.base_pos.unwrap_or((pos.x, pos.y));
                    match app_state.active_animation {
                        AppAnimation::Bounce => {
                            let mut nx = pos.x as f32 + app_state.bounce_vel_x;
                            let mut ny = pos.y as f32 + app_state.bounce_vel_y;
                            if nx < 0.0 { nx = 0.0; app_state.bounce_vel_x *= -1.0; }
                            else if nx + sz.width as f32 > mz.width as f32 { nx = mz.width as f32 - sz.width as f32; app_state.bounce_vel_x *= -1.0; }
                            if ny < 0.0 { ny = 0.0; app_state.bounce_vel_y *= -1.0; }
                            else if ny + sz.height as f32 > mz.height as f32 { ny = mz.height as f32 - sz.height as f32; app_state.bounce_vel_y *= -1.0; }
                            window.set_outer_position(winit::dpi::PhysicalPosition::new(nx as i32, ny as i32));
                            app_state.base_pos = Some((nx as i32, ny as i32));
                        }
                        AppAnimation::Shake => {
                            let i = 12.0_f32;
                            window.set_outer_position(winit::dpi::PhysicalPosition::new(bx + (app_state.anim_progress * 130.0).sin() as i32 * i as i32, by + (app_state.anim_progress * 115.0).cos() as i32 * i as i32));
                        }
                        AppAnimation::Dance => {
                            let r = 70.0_f32;
                            window.set_outer_position(winit::dpi::PhysicalPosition::new(bx + ((app_state.anim_progress * 4.0).sin() * r) as i32, by + ((app_state.anim_progress * 2.5).cos() * r) as i32));
                        }
                        AppAnimation::Dissolve => {
                            #[cfg(windows)]
                            if let Ok(h) = window.window_handle() { if let winit::raw_window_handle::RawWindowHandle::Win32(w32) = h.as_raw() {
                                let hwnd = HWND(w32.hwnd.get() as *mut _);
                                let op = 0.4 + 0.6 * (app_state.anim_progress * 2.5).cos().abs();
                                unsafe { let ex = GetWindowLongW(hwnd, GWL_EXSTYLE); if (ex & WS_EX_LAYERED.0 as i32) == 0 { windows::Win32::UI::WindowsAndMessaging::SetWindowLongW(hwnd, GWL_EXSTYLE, ex | WS_EX_LAYERED.0 as i32); } let _ = SetLayeredWindowAttributes(hwnd, None, (op * 255.0) as u8, LWA_ALPHA); }
                            }}
                        }
                        AppAnimation::Fly => {
                            let mut nx = pos.x as f32 + 12.0;
                            if nx > mz.width as f32 { nx = -(sz.width as f32); }
                            window.set_outer_position(winit::dpi::PhysicalPosition::new(nx as i32, (mz.height as f32 / 2.0 + (app_state.anim_progress * 2.0).sin() * 150.0) as i32));
                        }
                        _ => {}
                    }
                    window.request_redraw();
                }
            } else if app_state.base_pos.is_some() {
                #[cfg(windows)]
                if let Ok(h) = window.window_handle() { if let winit::raw_window_handle::RawWindowHandle::Win32(w32) = h.as_raw() { let hwnd = HWND(w32.hwnd.get() as *mut _); unsafe { let _ = SetLayeredWindowAttributes(hwnd, None, 255, LWA_ALPHA); } } }
                if matches!(app_state.active_animation, AppAnimation::Shake | AppAnimation::Dance) {
                    if let Some((x, y)) = app_state.base_pos { window.set_outer_position(winit::dpi::PhysicalPosition::new(x, y)); }
                }
                app_state.base_pos = None;
                app_state.anim_progress = 0.0;
            }

            // ── quote rotation timer ───────────────────────────────
            if app_state.rotation_enabled && !app_state.quotes.is_empty() {
                let secs = app_state.current_quote().and_then(|q| q.interval_secs).unwrap_or(app_state.interval_secs).clamp(1, 60);
                if app_state.last_rotation.elapsed() >= Duration::from_secs(secs) {
                    app_state.next_quote();
                }
            }

            // ── smooth rotation animation ──────────────────────────
            let needs_anim = (app_state.current_rotation_angle - app_state.target_rotation_angle).abs() > 0.01
                          || (app_state.current_scale - 1.0).abs() > 0.01;
            if needs_anim {
                let lerp = 1.0 - (-8.0_f32 * 0.016).exp();
                app_state.current_rotation_angle += (app_state.target_rotation_angle - app_state.current_rotation_angle) * lerp;
                let a = app_state.current_rotation_angle;
                let (w, h) = (content_rect.width(), content_rect.height());
                let target_scale = (w / (w * a.cos().abs() + h * a.sin().abs())).min(h / (w * a.sin().abs() + h * a.cos().abs())).min(1.0);
                app_state.current_scale += (target_scale - app_state.current_scale) * lerp;
                if (app_state.current_rotation_angle - app_state.target_rotation_angle).abs() > 0.01 { window.request_redraw(); }
            }

            // ── main content ───────────────────────────────────────
            let mut shaper = match (font_system.as_mut(), swash_cache.as_mut()) {
                (Some(fs), Some(sc)) => Some((fs, sc, &mut tex_cache)),
                _ => None,
            };
            render_main_content(ctx, app_state, window, &mut shaper);
            render_theme_modal(ctx, app_state);
            render_profile_modal(ctx, app_state);

            let float_actions = render_floating_buttons(ctx, app_state);
            for action in float_actions {
                match action {
                    TitleBarAction::TogglePanel => app_state.title_bar_state.control_panel_visible = !app_state.title_bar_state.control_panel_visible,
                    TitleBarAction::ShowHeader  => app_state.title_bar_state.header_visible = true,
                    _ => {}
                }
            }
        }); // end egui::run

        egui_state.handle_platform_output(window, full_output.platform_output);

        // ── shape transform for content rotation ───────────────────
        let shapes = if app_state.current_rotation_angle.abs() > 0.0001
            || (app_state.current_scale - 1.0).abs() > 0.0001
        {
            transform_content_shapes(
                &full_output.shapes, content_rect,
                app_state.current_rotation_angle,
                app_state.current_scale,
            )
        } else {
            full_output.shapes
        };

        // ── tessellate ─────────────────────────────────────────────
        let paint_jobs = egui_ctx.tessellate(shapes, scale);

        // ── CPU render — no GPU, no wgpu ───────────────────────────
        let bg = app_state.get_background_color();
        render_state.render(&paint_jobs, &full_output.textures_delta, scale, bg);

        // restore cosmic-text
        self.font_system = font_system;
        self.swash_cache = swash_cache;
        self.shaped_text_textures = tex_cache;
    }
}
