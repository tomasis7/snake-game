//! Minimal 2D vector for world coordinates.

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct V2 {
    pub x: f64,
    pub y: f64,
}

pub fn v2(x: f64, y: f64) -> V2 {
    V2 { x, y }
}
