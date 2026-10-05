//! Window-independent input state.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    Up,
    Down,
    Left,
    Right,
    W,
    A,
    S,
    D,
}

/// One-shot key presses (convenience keys, not part of the TS game).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tap {
    Num1,
    Num2,
    Enter,
    Escape,
    H,
    N,
    R,
    M,
    G,
}

#[derive(Default)]
pub struct Input {
    held: [bool; 8],
    pub mouse_x: f64,
    pub mouse_y: f64,
    pub mouse_down: bool,
    pub taps: Vec<Tap>,
}

impl Input {
    pub fn set_key(&mut self, k: Key, down: bool) {
        self.held[k as usize] = down;
    }
    pub fn key_down(&self, k: Key) -> bool {
        self.held[k as usize]
    }
    pub fn tapped(&self, t: Tap) -> bool {
        self.taps.contains(&t)
    }
}
