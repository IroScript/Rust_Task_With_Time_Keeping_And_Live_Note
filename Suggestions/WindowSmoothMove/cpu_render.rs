// cpu_render.rs
// =============================================================
// Pure-CPU rendering for the main window using `softbuffer`.
//
// Pipeline:
//   egui tessellation → ClippedPrimitive[]
//   → software triangle rasteriser (this file)
//   → XRGB8888 pixel buffer
//   → softbuffer → OS compositor (no GPU involvement)
//
// The background process (quantum_logo.exe) remains GPU-based.
// Only this window uses CPU rendering.
//
// CARGO.TOML additions required:
//   softbuffer = "0.4"
//
// (tiny-skia is NOT needed – we ship a hand-rolled scanline
//  rasteriser that handles egui's coloured + font-atlas meshes.)
// =============================================================

use std::{
    collections::HashMap,
    num::NonZeroU32,
    sync::Arc,
};

use egui::{
    epaint::{ClippedPrimitive, ImageDelta, Primitive},
    Color32, TextureId, TexturesDelta,
};
use winit::window::Window;

// ------------------------------------------------------------------
// Internal texture store
// ------------------------------------------------------------------
struct CpuTex {
    data: Vec<u32>,   // premultiplied ARGB8888  (alpha << 24 | R << 16 | G << 8 | B)
    w: u32,
    h: u32,
}

impl CpuTex {
    fn sample_nearest(&self, u: f32, v: f32) -> u32 {
        let px = (u * self.w as f32) as i32;
        let py = (v * self.h as f32) as i32;
        let px = px.clamp(0, self.w as i32 - 1) as u32;
        let py = py.clamp(0, self.h as i32 - 1) as u32;
        self.data[(py * self.w + px) as usize]
    }
}

// ------------------------------------------------------------------
// CpuRenderState
// ------------------------------------------------------------------
pub struct CpuRenderState {
    context: softbuffer::Context<Arc<Window>>,
    surface: softbuffer::Surface<Arc<Window>, Arc<Window>>,
    width:  u32,
    height: u32,
    textures: HashMap<TextureId, CpuTex>,
    // scratch pixel buffer (XRGB8888 for softbuffer)
    pixels: Vec<u32>,
}

impl CpuRenderState {
    pub fn new(window: Arc<Window>) -> Result<Self, String> {
        let context = softbuffer::Context::new(window.clone())
            .map_err(|e| format!("softbuffer context: {e}"))?;
        let mut surface = softbuffer::Surface::new(&context, window.clone())
            .map_err(|e| format!("softbuffer surface: {e}"))?;

        let sz = window.inner_size();
        let w = sz.width.max(1);
        let h = sz.height.max(1);
        surface.resize(NonZeroU32::new(w).unwrap(), NonZeroU32::new(h).unwrap())
               .map_err(|e| format!("resize: {e}"))?;

        Ok(Self {
            context,
            surface,
            width: w,
            height: h,
            textures: HashMap::new(),
            pixels: vec![0u32; (w * h) as usize],
        })
    }

    pub fn resize(&mut self, w: u32, h: u32) {
        let w = w.max(1);
        let h = h.max(1);
        if w == self.width && h == self.height { return; }
        self.width  = w;
        self.height = h;
        let _ = self.surface.resize(
            NonZeroU32::new(w).unwrap(),
            NonZeroU32::new(h).unwrap(),
        );
        self.pixels.resize((w * h) as usize, 0u32);
    }

    // ------------------------------------------------------------------
    // Main render entry point – called every frame
    // ------------------------------------------------------------------
    pub fn render(
        &mut self,
        paint_jobs:     &[ClippedPrimitive],
        textures_delta: &TexturesDelta,
        scale:          f32,
        bg:             Color32,
    ) {
        // 1. Update texture atlas
        for (id, delta) in &textures_delta.set {
            self.upload_texture(*id, delta);
        }

        // 2. Clear with background colour (XRGB)
        let bg_px = xrgb(bg.r(), bg.g(), bg.b());
        self.pixels.fill(bg_px);

        // 3. Rasterise every clipped primitive
        for prim in paint_jobs {
            if let Primitive::Mesh(mesh) = &prim.primitive {
                let clip = prim.clip_rect;
                let cx0 = (clip.min.x * scale).max(0.0) as i32;
                let cy0 = (clip.min.y * scale).max(0.0) as i32;
                let cx1 = (clip.max.x * scale).min(self.width  as f32) as i32;
                let cy1 = (clip.max.y * scale).min(self.height as f32) as i32;

                let tex = self.textures.get(&mesh.texture_id);

                for tri in mesh.indices.chunks_exact(3) {
                    let v0 = &mesh.vertices[tri[0] as usize];
                    let v1 = &mesh.vertices[tri[1] as usize];
                    let v2 = &mesh.vertices[tri[2] as usize];

                    rasterise_triangle(
                        v0, v1, v2,
                        scale,
                        cx0, cy0, cx1, cy1,
                        tex,
                        &mut self.pixels,
                        self.width,
                        self.height,
                    );
                }
            }
        }

        // 4. Blit pixel buffer → softbuffer
        if let Ok(mut buf) = self.surface.buffer_mut() {
            buf.copy_from_slice(&self.pixels);
            let _ = buf.present();
        }

        // 5. Free deleted textures
        for id in &textures_delta.free {
            self.textures.remove(id);
        }
    }

    // ------------------------------------------------------------------
    // Texture upload – handles both Color and Font (glyph atlas) images
    // ------------------------------------------------------------------
    fn upload_texture(&mut self, id: TextureId, delta: &ImageDelta) {
        use egui::ImageData;

        let [iw, ih] = delta.image.size();
        let iw = iw as u32;
        let ih = ih as u32;

        // Prepare ARGB data from egui image
        let argb: Vec<u32> = match &delta.image {
            ImageData::Color(img) => img
                .pixels
                .iter()
                .map(|c| argb(c.r(), c.g(), c.b(), c.a()))
                .collect(),

            ImageData::Font(fnt) => {
                // Font atlas is greyscale coverage; multiply with white
                fnt.srgba_pixels(None)
                   .map(|c| argb(c.r(), c.g(), c.b(), c.a()))
                   .collect()
            }
        };

        if let Some(pos) = delta.pos {
            // Partial update
            if let Some(tex) = self.textures.get_mut(&id) {
                let [ox, oy] = [pos[0] as u32, pos[1] as u32];
                for row in 0..ih {
                    for col in 0..iw {
                        let src = (row * iw + col) as usize;
                        let dst = ((oy + row) * tex.w + (ox + col)) as usize;
                        if dst < tex.data.len() {
                            tex.data[dst] = argb[src];
                        }
                    }
                }
                return;
            }
        }

        // Full replacement
        self.textures.insert(id, CpuTex { data: argb, w: iw, h: ih });
    }
}

// ------------------------------------------------------------------
// Software triangle rasteriser
// Uses barycentric scanline fill with per-pixel UV sampling and
// alpha blending.  Fast enough for typical egui scenes.
// ------------------------------------------------------------------
#[inline(always)]
fn rasterise_triangle(
    v0: &egui::epaint::Vertex,
    v1: &egui::epaint::Vertex,
    v2: &egui::epaint::Vertex,
    scale: f32,
    cx0: i32, cy0: i32, cx1: i32, cy1: i32,
    tex: Option<&CpuTex>,
    pixels: &mut [u32],
    pw: u32,
    ph: u32,
) {
    // Scale positions to physical pixels
    let (ax, ay) = (v0.pos.x * scale, v0.pos.y * scale);
    let (bx, by) = (v1.pos.x * scale, v1.pos.y * scale);
    let (cx, cy) = (v2.pos.x * scale, v2.pos.y * scale);

    // Bounding box (integer, clamped to clip + window)
    let min_x = ax.min(bx).min(cx).floor() as i32;
    let max_x = ax.max(bx).max(cx).ceil()  as i32;
    let min_y = ay.min(by).min(cy).floor() as i32;
    let max_y = ay.max(by).max(cy).ceil()  as i32;

    let min_x = min_x.max(cx0).max(0);
    let max_x = max_x.min(cx1 - 1).min(pw as i32 - 1);
    let min_y = min_y.max(cy0).max(0);
    let max_y = max_y.min(cy1 - 1).min(ph as i32 - 1);

    if min_x > max_x || min_y > max_y { return; }

    // Signed area × 2 of the triangle (for edge functions)
    let denom = edge(ax, ay, bx, by, cx, cy);
    if denom.abs() < 0.5 { return; }     // degenerate
    let inv_denom = 1.0 / denom;

    for py in min_y..=max_y {
        let pf_y = py as f32 + 0.5;
        for px in min_x..=max_x {
            let pf_x = px as f32 + 0.5;

            // Barycentric weights
            let w0 = edge(bx, by, cx, cy, pf_x, pf_y) * inv_denom;
            let w1 = edge(cx, cy, ax, ay, pf_x, pf_y) * inv_denom;
            let w2 = 1.0 - w0 - w1;

            if w0 < 0.0 || w1 < 0.0 || w2 < 0.0 { continue; }

            // Interpolate colour
            let r = (v0.color.r() as f32 * w0 + v1.color.r() as f32 * w1 + v2.color.r() as f32 * w2) as u8;
            let g = (v0.color.g() as f32 * w0 + v1.color.g() as f32 * w1 + v2.color.g() as f32 * w2) as u8;
            let b = (v0.color.b() as f32 * w0 + v1.color.b() as f32 * w1 + v2.color.b() as f32 * w2) as u8;
            let a = (v0.color.a() as f32 * w0 + v1.color.a() as f32 * w1 + v2.color.a() as f32 * w2) as u8;

            // Modulate with texture sample (font atlas or white pixel)
            let (tr, tg, tb, ta) = if let Some(t) = tex {
                let u = v0.uv.x * w0 + v1.uv.x * w1 + v2.uv.x * w2;
                let v = v0.uv.y * w0 + v1.uv.y * w1 + v2.uv.y * w2;
                let s = t.sample_nearest(u, v);
                (
                    ((s >> 16) & 0xFF) as u8,
                    ((s >>  8) & 0xFF) as u8,
                    ( s        & 0xFF) as u8,
                    ((s >> 24) & 0xFF) as u8,
                )
            } else {
                (r, g, b, a)
            };

            // Combine vertex colour × texel colour (both in [0,255])
            let fr = mul_u8(r, tr);
            let fg = mul_u8(g, tg);
            let fb = mul_u8(b, tb);
            let fa = mul_u8(a, ta);

            if fa == 0 { continue; }

            // Alpha-blend over destination
            let idx = (py * pw as i32 + px) as usize;
            let dst = pixels[idx];
            let dr = ((dst >> 16) & 0xFF) as u8;
            let dg = ((dst >>  8) & 0xFF) as u8;
            let db = ( dst        & 0xFF) as u8;

            let inv_a = 255 - fa as u32;
            let out_r = ((fr as u32 * fa as u32 + dr as u32 * inv_a) / 255) as u8;
            let out_g = ((fg as u32 * fa as u32 + dg as u32 * inv_a) / 255) as u8;
            let out_b = ((fb as u32 * fa as u32 + db as u32 * inv_a) / 255) as u8;

            pixels[idx] = xrgb(out_r, out_g, out_b);
        }
    }
}

// ------------------------------------------------------------------
// Helpers
// ------------------------------------------------------------------
#[inline(always)] fn edge(ax: f32, ay: f32, bx: f32, by: f32, px: f32, py: f32) -> f32 {
    (bx - ax) * (py - ay) - (by - ay) * (px - ax)
}
#[inline(always)] fn mul_u8(a: u8, b: u8) -> u8 { ((a as u32 * b as u32) / 255) as u8 }
#[inline(always)] fn xrgb(r: u8, g: u8, b: u8) -> u32 {
    0xFF00_0000 | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}
#[inline(always)] fn argb(r: u8, g: u8, b: u8, a: u8) -> u32 {
    ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}
