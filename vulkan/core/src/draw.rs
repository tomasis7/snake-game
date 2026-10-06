//! Painter: p5-style drawing calls expressed as instance quads. Draw order is the
//! order instances are written. No Vulkan here, only the instance records.

use crate::atlas::AtlasUv;
use crate::instance::*;
use crate::text::{self, HAlign};

pub type Color = [f32; 4];

pub fn rgba(hex: u32, a: f32) -> Color {
    [((hex >> 16) & 255) as f32 / 255.0, ((hex >> 8) & 255) as f32 / 255.0, (hex & 255) as f32 / 255.0, a]
}

pub fn rgb(hex: u32) -> Color {
    rgba(hex, 1.0)
}

pub struct Painter<'a> {
    out: &'a mut [Instance],
    pub n: usize,
    uv: &'a AtlasUv,
}

impl<'a> Painter<'a> {
    pub fn new(out: &'a mut [Instance], uv: &'a AtlasUv) -> Self {
        Painter { out, n: 0, uv }
    }

    #[inline]
    pub fn push(&mut self, i: Instance) {
        if self.n < self.out.len() {
            self.out[self.n] = i;
            self.n += 1;
        }
    }

    /// Flat rect, `(x, y)` top-left (p5 rectMode CORNER).
    pub fn solid(&mut self, x: f64, y: f64, w: f64, h: f64, color: Color) {
        self.push(Instance {
            pos: [x as f32, y as f32],
            size: [w as f32, h as f32],
            color,
            uv: [0.0; 4],
            params: [KIND_SOLID, 0.0, 0.0, 0.0],
        });
    }

    /// Flat rect centred on `(cx, cy)` (rectMode CENTER).
    pub fn solid_center(&mut self, cx: f64, cy: f64, w: f64, h: f64, color: Color) {
        self.solid(cx - w / 2.0, cy - h / 2.0, w, h, color);
    }

    pub fn rounded(&mut self, x: f64, y: f64, w: f64, h: f64, radius: f64, color: Color) {
        self.push(Instance {
            pos: [x as f32, y as f32],
            size: [w as f32, h as f32],
            color,
            uv: [0.0; 4],
            params: [KIND_ROUNDED, radius as f32, 0.0, 0.0],
        });
    }

    pub fn rounded_center(&mut self, cx: f64, cy: f64, w: f64, h: f64, radius: f64, color: Color) {
        self.rounded(cx - w / 2.0, cy - h / 2.0, w, h, radius, color);
    }

    /// 2 px outline of a rect centred on `(cx, cy)`, like `noFill(); stroke(255); strokeWeight(2)`.
    pub fn square_outline_center(&mut self, cx: f64, cy: f64, size: f64, weight: f64, color: Color) {
        let h = size / 2.0 + weight / 2.0;
        let inner = size - weight;
        self.solid(cx - h, cy - h, size + weight, weight, color);
        self.solid(cx - h, cy + size / 2.0 - weight / 2.0, size + weight, weight, color);
        self.solid(cx - h, cy - inner / 2.0, weight, inner, color);
        self.solid(cx + size / 2.0 - weight / 2.0, cy - inner / 2.0, weight, inner, color);
    }

    fn textured(&mut self, x: f64, y: f64, w: f64, h: f64, uv: [f32; 4], tint: Color) {
        self.push(Instance {
            pos: [x as f32, y as f32],
            size: [w as f32, h as f32],
            color: tint,
            uv,
            params: [KIND_TEXTURED, 0.0, 0.0, 0.0],
        });
    }

    /// The background image stretched over `(x, y, w, h)`.
    pub fn background(&mut self, x: f64, y: f64, w: f64, h: f64) {
        let uv = self.uv.background;
        self.textured(x, y, w, h, uv, [1.0; 4]);
    }

    /// Sprite drawn centred on `(cx, cy)` at `w x h` (drawSprite with imageMode CENTER).
    pub fn sprite_center(&mut self, id: usize, cx: f64, cy: f64, w: f64, h: f64) {
        let uv = self.uv.sprites[id];
        self.textured(cx - w / 2.0, cy - h / 2.0, w, h, uv, [1.0; 4]);
    }

    /// p5 `text()`: glyph squares of `size`, anchored by `align` and vertically centred.
    pub fn text(&mut self, s: &str, x: f64, y: f64, size: f64, color: Color, align: HAlign) {
        for g in text::layout(s, x, y, size, align) {
            let uv = self.uv.glyphs[g.index];
            self.textured(g.x, g.y, g.size, g.size, uv, color);
        }
    }

    /// Snake segment: radial gradient ball. `mid` is the 0.3 stop, `edge` the 1.0 stop.
    pub fn ball(&mut self, cx: f64, cy: f64, diameter: f64, mid: Color, edge: Color, centre_mix: f32) {
        self.push(Instance {
            pos: [(cx - diameter / 2.0) as f32, (cy - diameter / 2.0) as f32],
            size: [diameter as f32, diameter as f32],
            color: mid,
            uv: edge,
            params: [KIND_BALL, centre_mix, diameter as f32, 0.0],
        });
    }

    /// Soft round shadow centred on `(cx, cy)`.
    pub fn shadow(&mut self, cx: f64, cy: f64, diameter: f64, color: Color) {
        self.push(Instance {
            pos: [(cx - diameter / 2.0) as f32, (cy - diameter / 2.0) as f32],
            size: [diameter as f32, diameter as f32],
            color,
            uv: [0.0; 4],
            params: [KIND_SHADOW, 0.0, 0.0, 0.0],
        });
    }
}
