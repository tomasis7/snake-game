//! Text layout for the p5 `text()` equivalent: monospaced glyph squares.
//! Each glyph is a square of `size` canvas pixels with an advance of `size`.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HAlign {
    Left,
    Center,
    Right,
}

/// Glyph index into the atlas: 0..96 are ASCII 32..127, then the arrows the
/// pixel font lacks (the TS game falls back to a system font for those).
pub const GLYPH_COUNT: usize = 100;

pub fn glyph_index(c: char) -> usize {
    match c {
        ' '..='\u{7f}' => c as usize - 32,
        '←' => 96,
        '↑' => 97,
        '↓' => 98,
        '→' => 99,
        _ => '?' as usize - 32,
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Glyph {
    pub x: f64,
    pub y: f64,
    pub size: f64,
    pub index: usize,
}

/// p5's default text leading is 1.25 x the text size.
pub const LEADING: f64 = 1.25;

/// Lays out `text` (with `\n` line breaks). (x, y) is the anchor: horizontally by
/// `halign`, vertically the centre of the text block (CENTER).
pub fn layout(text: &str, x: f64, y: f64, size: f64, halign: HAlign) -> Vec<Glyph> {
    let lines: Vec<&str> = text.split('\n').collect();
    let line_h = size * LEADING;
    let block_h = size + line_h * (lines.len() as f64 - 1.0);
    let top = y - block_h / 2.0;
    let mut out = Vec::new();
    for (li, line) in lines.iter().enumerate() {
        let n = line.chars().count() as f64;
        let w = n * size;
        let left = match halign {
            HAlign::Left => x,
            HAlign::Center => x - w / 2.0,
            HAlign::Right => x - w,
        };
        let gy = top + li as f64 * line_h;
        for (i, c) in line.chars().enumerate() {
            if c == ' ' {
                continue;
            }
            out.push(Glyph { x: left + i as f64 * size, y: gy, size, index: glyph_index(c) });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indices() {
        assert_eq!(glyph_index(' '), 0);
        assert_eq!(glyph_index('A'), 33);
        assert_eq!(glyph_index('→'), 99);
        assert_eq!(glyph_index('é'), glyph_index('?'));
    }

    #[test]
    fn center_alignment() {
        let g = layout("AB", 100.0, 50.0, 10.0, HAlign::Center);
        assert_eq!(g.len(), 2);
        assert_eq!(g[0].x, 90.0);
        assert_eq!(g[1].x, 100.0);
        assert_eq!(g[0].y, 45.0);
    }

    #[test]
    fn left_right_alignment() {
        assert_eq!(layout("AB", 100.0, 0.0, 10.0, HAlign::Left)[0].x, 100.0);
        assert_eq!(layout("AB", 100.0, 0.0, 10.0, HAlign::Right)[0].x, 80.0);
    }

    #[test]
    fn spaces_skipped_but_advance() {
        let g = layout("A B", 0.0, 0.0, 8.0, HAlign::Left);
        assert_eq!(g.len(), 2);
        assert_eq!(g[1].x, 16.0);
    }

    #[test]
    fn multiline_centered_block() {
        let g = layout("A\nB", 0.0, 100.0, 10.0, HAlign::Left);
        assert_eq!(g[1].y - g[0].y, 12.5);
        let mid = (g[0].y + g[1].y + 10.0) / 2.0;
        assert!((mid - 100.0).abs() < 1e-9);
    }
}
