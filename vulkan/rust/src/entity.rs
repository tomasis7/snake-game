//! Level entities (block, tetris, winBlock, star, heart, plant, ghost) as one struct.

use crate::vec2::{v2, V2};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Block,
    Star,
    Heart,
    Plant,
    Ghost,
    Tetris,
    WinBlock,
}

impl Kind {
    /// Level tile code -> kind (codes 1..=7 as in levelfactory.ts).
    pub fn from_code(code: u8) -> Option<Kind> {
        Some(match code {
            1 => Kind::Block,
            2 => Kind::Star,
            3 => Kind::Heart,
            4 => Kind::Plant,
            5 => Kind::Ghost,
            6 => Kind::Tetris,
            7 => Kind::WinBlock,
            _ => return None,
        })
    }

    pub fn is_hazard(self) -> bool {
        matches!(self, Kind::Block | Kind::Tetris | Kind::Plant | Kind::Ghost)
    }
}

#[derive(Clone, Debug)]
pub struct Entity {
    pub kind: Kind,
    /// Drawn centred here; collisions treat it as the top-left corner.
    pub pos: V2,
    pub size: V2,
    pub vel: V2,
    pub removed: bool,
    pub sound_playing: bool,
    pub pulse_scale: f64,
}

impl Entity {
    pub fn new(kind: Kind, x: f64, y: f64) -> Entity {
        let size = match kind {
            Kind::Plant => v2(32.0, 64.0),
            Kind::Ghost => v2(50.0, 50.0),
            _ => v2(32.0, 32.0),
        };
        let vel = if kind == Kind::Ghost { v2(0.3, 0.3) } else { v2(0.0, 0.0) };
        Entity { kind, pos: v2(x, y), size, vel, removed: false, sound_playing: false, pulse_scale: 1.0 }
    }

    /// Per-frame update; `clock_ms` is the game clock (TS `millis()`).
    pub fn update(&mut self, clock_ms: f64) {
        match self.kind {
            Kind::Heart => {
                self.pulse_scale = 1.0 + 0.1 * (clock_ms * 0.01).sin();
            }
            Kind::Ghost => {
                self.pos.x += self.vel.x;
                self.pos.y += self.vel.y;
                if self.pos.x < 0.0 || self.pos.x > 1200.0 {
                    self.vel.x *= -1.0;
                }
                if self.pos.y < 0.0 || self.pos.y > 800.0 {
                    self.vel.y *= -1.0;
                }
            }
            _ => {}
        }
    }
}
