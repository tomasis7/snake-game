//! CPU-built texture atlas: background, sprites (1 texel per sprite pixel, 1 texel
//! padding), font cells, extra arrow glyphs and one opaque white texel.

use std::io::Cursor;

use crate::sprites::{self, NAMES};
use crate::text::GLYPH_COUNT;

pub const ATLAS_SIZE: usize = 2048;

static BACKGROUND_PNG: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../assets/background.png"));
static FONT_PNG: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../assets/font.png"));

pub struct Decoded {
    pub width: usize,
    pub height: usize,
    pub rgba: Vec<u8>,
}

pub fn decode_png(bytes: &[u8]) -> Result<Decoded, String> {
    let mut dec = png::Decoder::new(Cursor::new(bytes));
    dec.set_transformations(png::Transformations::EXPAND);
    let mut reader = dec.read_info().map_err(|e| e.to_string())?;
    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).map_err(|e| e.to_string())?;
    let (w, h) = (info.width as usize, info.height as usize);
    let src = &buf[..info.buffer_size()];
    let mut rgba = Vec::with_capacity(w * h * 4);
    match (info.color_type, info.bit_depth) {
        (png::ColorType::Rgba, png::BitDepth::Eight) => rgba.extend_from_slice(src),
        (png::ColorType::Rgb, png::BitDepth::Eight) => {
            for p in src.chunks_exact(3) {
                rgba.extend_from_slice(&[p[0], p[1], p[2], 255]);
            }
        }
        (png::ColorType::Grayscale, png::BitDepth::Eight) => {
            for &g in src {
                rgba.extend_from_slice(&[g, g, g, 255]);
            }
        }
        (png::ColorType::GrayscaleAlpha, png::BitDepth::Eight) => {
            for p in src.chunks_exact(2) {
                rgba.extend_from_slice(&[p[0], p[0], p[0], p[1]]);
            }
        }
        other => return Err(format!("unsupported PNG format {other:?}")),
    }
    Ok(Decoded { width: w, height: h, rgba })
}

/// UV rectangles (u0 v0 u1 v1) of everything in the atlas.
#[derive(Clone)]
pub struct AtlasUv {
    pub background: [f32; 4],
    #[allow(dead_code)] // the opaque texel is part of the atlas layout; solids use KIND_SOLID
    pub white: [f32; 4],
    pub sprites: [[f32; 4]; NAMES.len()],
    pub glyphs: Vec<[f32; 4]>,
}

pub struct Atlas {
    pub rgba: Vec<u8>,
    pub uv: AtlasUv,
}

fn uv_rect(x: usize, y: usize, w: usize, h: usize) -> [f32; 4] {
    let s = ATLAS_SIZE as f32;
    [x as f32 / s, y as f32 / s, (x + w) as f32 / s, (y + h) as f32 / s]
}

fn blit(dst: &mut [u8], dx: usize, dy: usize, src: &Decoded) {
    for y in 0..src.height {
        let d = ((dy + y) * ATLAS_SIZE + dx) * 4;
        let s = y * src.width * 4;
        dst[d..d + src.width * 4].copy_from_slice(&src.rgba[s..s + src.width * 4]);
    }
}

/// Arrow glyphs the pixel font lacks: left, up, down, right (8 x 8, white).
fn arrow_cells() -> [[bool; 64]; 4] {
    const LEFT: [&str; 8] =
        ["........", "..#.....", ".##.....", "########", ".##.....", "..#.....", "........", "........"];
    let mut left = [false; 64];
    for (y, row) in LEFT.iter().enumerate() {
        for (x, c) in row.bytes().enumerate() {
            left[y * 8 + x] = c == b'#';
        }
    }
    let (mut right, mut up, mut down) = ([false; 64], [false; 64], [false; 64]);
    for y in 0..8 {
        for x in 0..8 {
            right[y * 8 + x] = left[y * 8 + (7 - x)];
            up[y * 8 + x] = left[x * 8 + y];
            down[y * 8 + x] = left[(7 - x) * 8 + y];
        }
    }
    [left, up, down, right]
}

pub fn build() -> Result<Atlas, String> {
    let mut rgba = vec![0u8; ATLAS_SIZE * ATLAS_SIZE * 4];

    let bg = decode_png(BACKGROUND_PNG)?;
    blit(&mut rgba, 0, 0, &bg);
    let background = uv_rect(0, 0, bg.width, bg.height);

    // Font: 16 x 6 cells of 8 x 8, then the arrow cells right after, below the background.
    let font_y = bg.height + 8;
    let font = decode_png(FONT_PNG)?;
    blit(&mut rgba, 0, font_y, &font);
    let mut glyphs = Vec::with_capacity(GLYPH_COUNT);
    for i in 0..96 {
        glyphs.push(uv_rect((i % 16) * 8, font_y + (i / 16) * 8, 8, 8));
    }
    let arrow_y = font_y + font.height + 8;
    for (k, cell) in arrow_cells().iter().enumerate() {
        for y in 0..8 {
            for x in 0..8 {
                if cell[y * 8 + x] {
                    let o = ((arrow_y + y) * ATLAS_SIZE + k * 8 + x) * 4;
                    rgba[o..o + 4].copy_from_slice(&[255, 255, 255, 255]);
                }
            }
        }
        glyphs.push(uv_rect(k * 8, arrow_y, 8, 8));
    }
    debug_assert_eq!(glyphs.len(), GLYPH_COUNT);

    // Sprites, shelf-packed below.
    let list = sprites::load();
    let mut sprite_uv = [[0.0f32; 4]; NAMES.len()];
    let (mut cx, mut cy, mut shelf_h) = (0usize, arrow_y + 16, 0usize);
    for (i, s) in list.iter().enumerate() {
        let (w, h) = (s.cols + 2, s.rows.len() + 2);
        if cx + w > ATLAS_SIZE {
            cx = 0;
            cy += shelf_h;
            shelf_h = 0;
        }
        if cy + h > ATLAS_SIZE - 4 {
            return Err("atlas full".into());
        }
        for (y, row) in s.rows.iter().enumerate() {
            for (x, c) in row.chars().enumerate() {
                if c == '.' {
                    continue;
                }
                let col = s.palette.get(&c).copied().unwrap_or(0xff00ff);
                let o = ((cy + 1 + y) * ATLAS_SIZE + cx + 1 + x) * 4;
                rgba[o..o + 4].copy_from_slice(&[(col >> 16) as u8, (col >> 8) as u8, col as u8, 255]);
            }
        }
        sprite_uv[i] = uv_rect(cx + 1, cy + 1, s.cols, s.rows.len());
        cx += w;
        shelf_h = shelf_h.max(h);
    }

    // One opaque white texel in the bottom-right corner.
    let o = ((ATLAS_SIZE - 1) * ATLAS_SIZE + ATLAS_SIZE - 1) * 4;
    rgba[o..o + 4].copy_from_slice(&[255, 255, 255, 255]);
    let c = (ATLAS_SIZE as f32 - 0.5) / ATLAS_SIZE as f32;

    Ok(Atlas { rgba, uv: AtlasUv { background, white: [c, c, c, c], sprites: sprite_uv, glyphs } })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atlas_builds_and_fits() {
        let a = build().unwrap();
        assert_eq!(a.rgba.len(), ATLAS_SIZE * ATLAS_SIZE * 4);
        assert_eq!(a.uv.glyphs.len(), GLYPH_COUNT);
        // White texel is opaque white.
        let n = a.rgba.len();
        assert_eq!(&a.rgba[n - 4..], &[255, 255, 255, 255]);
        // Font 'A' has opaque pixels, space is empty.
        let cell_opaque = |uv: [f32; 4]| {
            let (x0, y0) = ((uv[0] * 2048.0) as usize, (uv[1] * 2048.0) as usize);
            (0..8).any(|y| (0..8).any(|x| a.rgba[((y0 + y) * ATLAS_SIZE + x0 + x) * 4 + 3] > 128))
        };
        assert!(cell_opaque(a.uv.glyphs[33]));
        assert!(!cell_opaque(a.uv.glyphs[0]));
        for k in 96..100 {
            assert!(cell_opaque(a.uv.glyphs[k]));
        }
    }
}
