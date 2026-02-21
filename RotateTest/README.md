# Rust WinAPI — Rotating & Bouncing Window

## Build & Run (Windows only)
```bash
cargo build --release
./target/release/rotating_window.exe
```

## Controls
| Key | Action |
|-----|--------|
| R   | Toggle window rotation (text + border rotates via GDI escapement) |
| B   | Toggle bounce (window flies around screen) |
| ESC | Quit |

## How "Window Rotation" Works

Windows OS **does NOT** provide a native API to rotate the entire window frame
(titlebar, borders, chrome). This is a hard OS-level limitation confirmed by:
- Raymond Chen (Microsoft, 30+ years WinAPI): https://devblogs.microsoft.com/oldnewthing/
- Microsoft Docs on DWM: https://learn.microsoft.com/en-us/windows/win32/dwm/dwm-overview

### The Solution Used Here:
1. **`WS_POPUP`** style → removes titlebar/borders entirely → clean canvas
2. **GDI `CreateFont` with `cEscapement`** → rotates all text rendered in the window
3. **`SetWindowPos`** in a 60fps timer → moves window for bounce physics
4. The border is drawn manually with `LineTo` → it also gets the rotation effect through the rotated DC

### True Pixel-Level Window Rotation (Advanced):
For pixel-perfect full rotation (like rotating a screenshot of the window):
- Use **Direct2D** `ID2D1RenderTarget::SetTransform()` with a rotation matrix
- Or use **DirectComposition** `IDCompositionRotateTransform`
- These rotate the *rendered content* not the OS window chrome

### Why Not SetWindowDisplayAffinity or DWM Thumbnail?
- `SetWindowDisplayAffinity` is for screen capture protection only
- DWM thumbnails are read-only projections
- No WinAPI exists for "rotate this HWND by N degrees" — this is confirmed limitation

## Architecture
```
WinMain loop
  ├── WM_TIMER (16ms / ~60fps)
  │     ├── increment angle (if rotating)
  │     └── update position + bounce physics (if bouncing)
  └── WM_PAINT
        ├── FillRect (background)
        ├── CreateFont(escapement=angle*10) → rotated text
        └── DrawBorder lines
```
