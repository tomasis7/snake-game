//! Letterboxing of the 1200 x 800 virtual canvas into the window (3:2).

pub const CANVAS_W: f32 = 1200.0;
pub const CANVAS_H: f32 = 800.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Letterbox {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Letterbox {
    /// Largest centred 3:2 rect (whole pixels) inside a window of the given size.
    pub fn fit(win_w: u32, win_h: u32) -> Letterbox {
        let scale = (win_w as f32 / CANVAS_W).min(win_h as f32 / CANVAS_H);
        let w = (CANVAS_W * scale).floor().max(1.0);
        let h = (CANVAS_H * scale).floor().max(1.0);
        Letterbox { x: ((win_w as f32 - w) / 2.0).floor(), y: ((win_h as f32 - h) / 2.0).floor(), w, h }
    }

    /// Window pixel -> canvas pixel.
    pub fn to_canvas(&self, px: f64, py: f64) -> (f64, f64) {
        (
            (px - self.x as f64) * CANVAS_W as f64 / self.w as f64,
            (py - self.y as f64) * CANVAS_H as f64 / self.h as f64,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_fit() {
        assert_eq!(Letterbox::fit(1200, 800), Letterbox { x: 0.0, y: 0.0, w: 1200.0, h: 800.0 });
    }

    #[test]
    fn wide_window_pillarboxes() {
        let l = Letterbox::fit(2000, 800);
        assert_eq!((l.w, l.h, l.x, l.y), (1200.0, 800.0, 400.0, 0.0));
    }

    #[test]
    fn tall_window_letterboxes() {
        let l = Letterbox::fit(600, 800);
        assert_eq!((l.w, l.h, l.x), (600.0, 400.0, 0.0));
        assert_eq!(l.y, 200.0);
    }

    #[test]
    fn mouse_mapping() {
        let l = Letterbox::fit(600, 800);
        assert_eq!(l.to_canvas(300.0, 400.0), (600.0, 400.0));
    }
}
