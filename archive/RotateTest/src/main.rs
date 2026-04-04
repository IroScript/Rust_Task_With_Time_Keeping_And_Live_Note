#![windows_subsystem = "windows"]

use std::mem;
use std::sync::Mutex;

// Windows API bindings (raw unsafe)
#[link(name = "user32")]
extern "system" {
    fn CreateWindowExW(
        dwExStyle: u32,
        lpClassName: *const u16,
        lpWindowName: *const u16,
        dwStyle: u32,
        x: i32,
        y: i32,
        nWidth: i32,
        nHeight: i32,
        hWndParent: *mut u8,
        hMenu: *mut u8,
        hInstance: *mut u8,
        lpParam: *mut u8,
    ) -> *mut u8;
    fn ShowWindow(hWnd: *mut u8, nCmdShow: i32) -> i32;
    fn UpdateWindow(hWnd: *mut u8) -> i32;
    fn GetMessageW(lpMsg: *mut MSG, hWnd: *mut u8, wMsgFilterMin: u32, wMsgFilterMax: u32) -> i32;
    fn TranslateMessage(lpMsg: *const MSG) -> i32;
    fn DispatchMessageW(lpMsg: *const MSG) -> isize;
    fn DefWindowProcW(hWnd: *mut u8, msg: u32, wParam: usize, lParam: isize) -> isize;
    fn RegisterClassExW(lpwcx: *const WNDCLASSEXW) -> u16;
    fn PostQuitMessage(nExitCode: i32);
    fn InvalidateRect(hWnd: *mut u8, lpRect: *mut u8, bErase: i32) -> i32;
    fn BeginPaint(hWnd: *mut u8, lpPaint: *mut PAINTSTRUCT) -> *mut u8;
    fn EndPaint(hWnd: *mut u8, lpPaint: *const PAINTSTRUCT) -> i32;
    fn GetClientRect(hWnd: *mut u8, lpRect: *mut RECT) -> i32;
    fn SetWindowPos(
        hWnd: *mut u8,
        hWndInsertAfter: *mut u8,
        x: i32,
        y: i32,
        cx: i32,
        cy: i32,
        uFlags: u32,
    ) -> i32;
    fn GetSystemMetrics(nIndex: i32) -> i32;
    fn SetTimer(hWnd: *mut u8, nIDEvent: usize, uElapse: u32, lpTimerFunc: *mut u8) -> usize;
}

#[link(name = "gdi32")]
extern "system" {
    fn SetBkMode(hdc: *mut u8, iBkMode: i32) -> i32;
    fn SetTextColor(hdc: *mut u8, color: u32) -> u32;
    fn TextOutW(hdc: *mut u8, x: i32, y: i32, lpString: *const u16, c: i32) -> i32;
    fn FillRect(hdc: *mut u8, lprc: *const RECT, hbr: *mut u8) -> i32;
    fn CreateSolidBrush(color: u32) -> *mut u8;
    fn DeleteObject(ho: *mut u8) -> i32;
    fn SelectObject(hdc: *mut u8, h: *mut u8) -> *mut u8;
    fn CreateFontW(
        cHeight: i32,
        cWidth: i32,
        cEscapement: i32,
        cOrientation: i32,
        cWeight: i32,
        bItalic: u32,
        bUnderline: u32,
        bStrikeOut: u32,
        iCharSet: u32,
        iOutPrecision: u32,
        iClipPrecision: u32,
        iQuality: u32,
        iPitchAndFamily: u32,
        pszFaceName: *const u16,
    ) -> *mut u8;
    fn MoveToEx(hdc: *mut u8, x: i32, y: i32, lppt: *mut u8) -> i32;
    fn LineTo(hdc: *mut u8, x: i32, y: i32) -> i32;
    fn CreatePen(iStyle: i32, cWidth: i32, color: u32) -> *mut u8;
}

#[link(name = "kernel32")]
extern "system" {
    fn GetModuleHandleW(lpModuleName: *const u16) -> *mut u8;
}

const WM_DESTROY: u32 = 0x0002;
const WM_PAINT: u32 = 0x000F;
const WM_KEYDOWN: u32 = 0x0100;
const WM_TIMER: u32 = 0x0113;
const WS_POPUP: u32 = 0x80000000;
const WS_EX_LAYERED: u32 = 0x00080000;
const WS_EX_TOPMOST: u32 = 0x00000008;
const SW_SHOW: i32 = 5;
const TRANSPARENT: i32 = 1;
const SWP_NOSIZE: u32 = 0x0001;
const SWP_NOZORDER: u32 = 0x0004;
const SM_CXSCREEN: i32 = 0;
const SM_CYSCREEN: i32 = 1;

#[repr(C)]
struct RECT {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

#[allow(non_snake_case)]
#[repr(C)]
struct PAINTSTRUCT {
    hdc: *mut u8,
    fErase: i32,
    rcPaint: RECT,
    fRestore: i32,
    fIncUpdate: i32,
    rgbReserved: [u8; 32],
}

#[allow(non_snake_case)]
#[repr(C)]
struct MSG {
    hwnd: *mut u8,
    message: u32,
    wParam: usize,
    lParam: isize,
    time: u32,
    pt_x: i32,
    pt_y: i32,
}

#[allow(non_snake_case)]
#[repr(C)]
struct WNDCLASSEXW {
    cbSize: u32,
    style: u32,
    lpfnWndProc: unsafe extern "system" fn(*mut u8, u32, usize, isize) -> isize,
    cbClsExtra: i32,
    cbWndExtra: i32,
    hInstance: *mut u8,
    hIcon: *mut u8,
    hCursor: *mut u8,
    hbrBackground: *mut u8,
    lpszMenuName: *const u16,
    lpszClassName: *const u16,
    hIconSm: *mut u8,
}

// Global state
struct AppState {
    angle: f64,     // rotation angle in degrees (0..360)
    bounce_x: f64,  // window center X
    bounce_y: f64,  // window center Y
    vel_x: f64,     // velocity X
    vel_y: f64,     // velocity Y
    rotating: bool, // is rotating?
    bouncing: bool, // is bouncing?
    win_w: i32,
    win_h: i32,
    screen_w: i32,
    screen_h: i32,
}

static STATE: Mutex<Option<AppState>> = Mutex::new(None);
static mut HWND_GLOBAL: *mut u8 = std::ptr::null_mut();

fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

unsafe extern "system" fn wnd_proc(hwnd: *mut u8, msg: u32, wparam: usize, lparam: isize) -> isize {
    match msg {
        WM_KEYDOWN => {
            let mut state_guard = STATE.lock().unwrap();
            if let Some(s) = state_guard.as_mut() {
                match wparam as u8 {
                    b'R' | b'r' => {
                        s.rotating = !s.rotating;
                    }
                    b'B' | b'b' => {
                        s.bouncing = !s.bouncing;
                        if s.bouncing {
                            s.vel_x = 4.0;
                            s.vel_y = 3.5;
                        }
                    }
                    0x1B => {
                        // ESC
                        PostQuitMessage(0);
                    }
                    _ => {}
                }
            }
            0
        }
        WM_TIMER => {
            let mut state_guard = STATE.lock().unwrap();
            if let Some(s) = state_guard.as_mut() {
                if s.rotating {
                    s.angle = (s.angle + 2.0) % 360.0;
                }

                if s.bouncing {
                    s.bounce_x += s.vel_x;
                    s.bounce_y += s.vel_y;
                    let hw = (s.win_w / 2) as f64;
                    let hh = (s.win_h / 2) as f64;
                    if s.bounce_x - hw < 0.0 || s.bounce_x + hw > s.screen_w as f64 {
                        s.vel_x = -s.vel_x;
                        s.bounce_x += s.vel_x;
                    }
                    if s.bounce_y - hh < 0.0 || s.bounce_y + hh > s.screen_h as f64 {
                        s.vel_y = -s.vel_y;
                        s.bounce_y += s.vel_y;
                    }
                    let nx = (s.bounce_x - hw) as i32;
                    let ny = (s.bounce_y - hh) as i32;
                    SetWindowPos(
                        hwnd,
                        std::ptr::null_mut(),
                        nx,
                        ny,
                        0,
                        0,
                        SWP_NOSIZE | SWP_NOZORDER,
                    );
                }
            }

            drop(state_guard);
            InvalidateRect(hwnd, std::ptr::null_mut(), 1);
            0
        }
        WM_PAINT => {
            let mut ps: PAINTSTRUCT = mem::zeroed();
            let hdc = BeginPaint(hwnd, &mut ps);

            let state_guard = STATE.lock().unwrap();
            if let Some(s) = state_guard.as_ref() {
                let mut rc: RECT = mem::zeroed();
                GetClientRect(hwnd, &mut rc);
                let w = rc.right;
                let h = rc.bottom;

                // Background - deep space black
                let bg_brush = CreateSolidBrush(0x001A0A2E); // dark navy
                FillRect(hdc, &rc, bg_brush);
                DeleteObject(bg_brush);

                // Draw border glow lines
                let pen = CreatePen(0, 3, 0x00FF6B6B); // coral red
                let old_pen = SelectObject(hdc, pen);
                MoveToEx(hdc, 2, 2, std::ptr::null_mut());
                LineTo(hdc, w - 2, 2);
                LineTo(hdc, w - 2, h - 2);
                LineTo(hdc, 2, h - 2);
                LineTo(hdc, 2, 2);
                SelectObject(hdc, old_pen);
                DeleteObject(pen);

                // Angle in degrees → escapement is in tenths of degrees for CreateFont
                let esc = (s.angle * 10.0) as i32;

                // Create rotated font
                let face = to_wide("Segoe UI");
                let font = CreateFontW(42, 0, esc, esc, 700, 0, 0, 0, 0, 0, 0, 0, 0, face.as_ptr());
                let old_font = SelectObject(hdc, font);

                SetBkMode(hdc, TRANSPARENT);

                // Neon cyan text
                SetTextColor(hdc, 0x00FFD700); // gold
                let line1 = to_wide("🚀 RUST + WINAPI DEMO");
                TextOutW(
                    hdc,
                    w / 2 - 160,
                    h / 2 - 60,
                    line1.as_ptr(),
                    (line1.len() - 1) as i32,
                );

                // Neon green sub text
                let small_face = to_wide("Consolas");
                let small_font = CreateFontW(
                    22,
                    0,
                    esc,
                    esc,
                    400,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    small_face.as_ptr(),
                );
                SelectObject(hdc, small_font);

                SetTextColor(hdc, 0x0000FF88); // neon green
                let line2 = to_wide("Press R = Rotate Window");
                TextOutW(
                    hdc,
                    w / 2 - 140,
                    h / 2 + 10,
                    line2.as_ptr(),
                    (line2.len() - 1) as i32,
                );

                let line3 = to_wide("Press B = Bounce Window");
                SetTextColor(hdc, 0x00FF8800); // orange
                TextOutW(
                    hdc,
                    w / 2 - 140,
                    h / 2 + 40,
                    line3.as_ptr(),
                    (line3.len() - 1) as i32,
                );

                let line4 = to_wide("Press ESC = Quit");
                SetTextColor(hdc, 0x00FF4466); // pink
                TextOutW(
                    hdc,
                    w / 2 - 100,
                    h / 2 + 70,
                    line4.as_ptr(),
                    (line4.len() - 1) as i32,
                );

                // Status
                let status_str = if s.rotating && s.bouncing {
                    "[ ROTATING + BOUNCING ]"
                } else if s.rotating {
                    "[ ROTATING ]"
                } else if s.bouncing {
                    "[ BOUNCING ]"
                } else {
                    "[ PRESS R or B ]"
                };

                let status_face = to_wide("Consolas");
                let status_font = CreateFontW(
                    18,
                    0,
                    0,
                    0,
                    700,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    status_face.as_ptr(),
                );
                SelectObject(hdc, status_font);
                SetTextColor(hdc, 0x00FFFFFF);
                let sw = to_wide(status_str);
                TextOutW(hdc, 10, 10, sw.as_ptr(), (sw.len() - 1) as i32);

                // Angle display
                let angle_str = format!("Angle: {:.0}°", s.angle);
                let aw = to_wide(&angle_str);
                SetTextColor(hdc, 0x00AAAAAA);
                TextOutW(hdc, 10, 32, aw.as_ptr(), (aw.len() - 1) as i32);

                SelectObject(hdc, old_font);
                DeleteObject(font);
                DeleteObject(small_font);
                DeleteObject(status_font);
            }

            EndPaint(hwnd, &ps);
            0
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

fn main() {
    unsafe {
        let screen_w = GetSystemMetrics(SM_CXSCREEN);
        let screen_h = GetSystemMetrics(SM_CYSCREEN);
        let win_w = 520i32;
        let win_h = 300i32;

        *STATE.lock().unwrap() = Some(AppState {
            angle: 0.0,
            bounce_x: (screen_w / 2) as f64,
            bounce_y: (screen_h / 2) as f64,
            vel_x: 4.0,
            vel_y: 3.5,
            rotating: false,
            bouncing: false,
            win_w,
            win_h,
            screen_w,
            screen_h,
        });

        let hinstance = GetModuleHandleW(std::ptr::null());
        let class_name = to_wide("RotatingWindowClass");

        let wc = WNDCLASSEXW {
            cbSize: mem::size_of::<WNDCLASSEXW>() as u32,
            style: 0x0003,
            lpfnWndProc: wnd_proc,
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: hinstance,
            hIcon: std::ptr::null_mut(),
            hCursor: std::ptr::null_mut(),
            hbrBackground: std::ptr::null_mut(),
            lpszMenuName: std::ptr::null(),
            lpszClassName: class_name.as_ptr(),
            hIconSm: std::ptr::null_mut(),
        };
        RegisterClassExW(&wc);

        let title = to_wide("Rust WinAPI — Rotating & Bouncing Window");
        let x = screen_w / 2 - win_w / 2;
        let y = screen_h / 2 - win_h / 2;

        // WS_POPUP = borderless window (so rotation looks clean)
        let hwnd = CreateWindowExW(
            WS_EX_TOPMOST, // Removed WS_EX_LAYERED to ensure visibility without extra setup
            class_name.as_ptr(),
            title.as_ptr(),
            WS_POPUP,
            x,
            y,
            win_w,
            win_h,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            hinstance,
            std::ptr::null_mut(),
        );
        HWND_GLOBAL = hwnd;

        ShowWindow(hwnd, SW_SHOW);
        UpdateWindow(hwnd);

        // Timer: 16ms ≈ 60fps
        SetTimer(hwnd, 1, 16, std::ptr::null_mut());

        let mut msg: MSG = mem::zeroed();
        loop {
            let ret = GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0);
            if ret <= 0 {
                break;
            }
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}
