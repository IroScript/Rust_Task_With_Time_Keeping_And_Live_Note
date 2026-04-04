// native_drag.rs
// ==============================================================
// Zero-latency window dragging via WM_NCHITTEST subclassing.
//
// HOW IT WORKS:
//   Windows sends WM_NCHITTEST BEFORE any mouse button events.
//   By returning HTCAPTION for the title-bar drag zone the OS
//   takes over the move loop itself — identical to Notepad/Chrome.
//   egui never even sees the click, so there is NO frame latency.
//
// PLACEMENT:
//   Add `mod native_drag;` at the top of main.rs.
//   Call `native_drag::install(hwnd)` right after window creation.
// ==============================================================

#![cfg(windows)]

use std::sync::atomic::{AtomicI32, AtomicIsize, Ordering};

use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::UI::HiDpi::GetDpiForWindow;
use windows::Win32::UI::WindowsAndMessaging::{
    CallWindowProcW, DefWindowProcW, GetClientRect, ScreenToClient, SetWindowLongPtrW,
    GWLP_WNDPROC,
};

// Windows hit-test constants (not always re-exported in every windows-rs version)
const WM_NCHITTEST: u32 = 0x0084;
const HTCLIENT: isize = 1;
const HTCAPTION: isize = 2;

// ------------------------------------------------------------------
// Adjustable zones (logical pixels).  Tune if buttons overlap drag.
// ------------------------------------------------------------------
/// Left dead-zone – skip the app icon / left-edge resize border.
pub static LEFT_DEAD_PX: AtomicI32 = AtomicI32::new(8);
/// Right dead-zone – all title-bar buttons live here.
/// 450 covers: close+max+min + hide + 6×anim + bg + zoom×2 + export + theme + profile + quote-toggle
pub static RIGHT_BUTTONS_PX: AtomicI32 = AtomicI32::new(450);
/// Height of the custom title bar (logical pixels).
pub static TITLE_BAR_H_PX: AtomicI32 = AtomicI32::new(26);

// ------------------------------------------------------------------
// Stored previous window procedure (must survive for the lifetime of
// the window, so we put it in a static).
// ------------------------------------------------------------------
static PREV_WNDPROC: AtomicIsize = AtomicIsize::new(0);

/// Custom window procedure.  Intercepts WM_NCHITTEST only; everything
/// else is forwarded to the original proc.
unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if msg == WM_NCHITTEST {
        // Ask Windows for the default result first.
        let default = DefWindowProcW(hwnd, msg, wparam, lparam);

        // Only upgrade HTCLIENT hits (the frameless window's content area).
        if default.0 == HTCLIENT {
            // lparam encodes the cursor's screen coordinates.
            // Low word = X,  high word = Y  (both signed 16-bit).
            let screen_x = (lparam.0 & 0xFFFF) as i16 as i32;
            let screen_y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;

            // Convert to client (window) coordinates.
            let mut pt = POINT { x: screen_x, y: screen_y };
            let _ = ScreenToClient(hwnd, &mut pt);

            // DPI-aware scaling factor (96 dpi = scale 1.0).
            let dpi = GetDpiForWindow(hwnd);
            let scale = dpi as f32 / 96.0;

            // Physical-pixel thresholds derived from logical-pixel constants.
            let title_h = TITLE_BAR_H_PX.load(Ordering::Relaxed) as f32 * scale;
            let left_dead = LEFT_DEAD_PX.load(Ordering::Relaxed) as f32 * scale;
            let right_btns = RIGHT_BUTTONS_PX.load(Ordering::Relaxed) as f32 * scale;

            // Client width.
            let mut rect = RECT::default();
            let _ = GetClientRect(hwnd, &mut rect);
            let client_w = rect.right as f32;

            // Is the cursor in the draggable title-bar strip?
            let in_titlebar = pt.y >= 0 && (pt.y as f32) < title_h;
            let in_drag_strip =
                (pt.x as f32) >= left_dead && (pt.x as f32) < (client_w - right_btns);

            if in_titlebar && in_drag_strip {
                return LRESULT(HTCAPTION);   // OS handles the entire drag — zero latency
            }
        }

        return default;
    }

    // Forward everything else to the original proc.
    let prev = PREV_WNDPROC.load(Ordering::Relaxed);
    if prev != 0 {
        // SAFETY: `prev` was obtained from SetWindowLongPtrW and is valid for this window.
        CallWindowProcW(
            Some(std::mem::transmute(prev as usize)),
            hwnd,
            msg,
            wparam,
            lparam,
        )
    } else {
        DefWindowProcW(hwnd, msg, wparam, lparam)
    }
}

/// Install the native drag handler on `hwnd`.
///
/// Call this **once**, immediately after window creation.
/// Safe to call from any thread as long as `hwnd` is valid.
pub fn install(hwnd: HWND) {
    let new_proc = wnd_proc as usize as isize;
    // SAFETY: hwnd is a valid window handle created by our process.
    let prev = unsafe { SetWindowLongPtrW(hwnd, GWLP_WNDPROC, new_proc) };
    PREV_WNDPROC.store(prev, Ordering::Relaxed);
    eprintln!("[native_drag] Installed WM_NCHITTEST handler → zero-latency drag active");
}

/// Uninstall the handler (call before the window is destroyed if needed).
pub fn uninstall(hwnd: HWND) {
    let prev = PREV_WNDPROC.load(Ordering::Relaxed);
    if prev != 0 {
        unsafe { SetWindowLongPtrW(hwnd, GWLP_WNDPROC, prev) };
    }
}
