//! Level loading (port of levelfactory.ts using the exported `assets/levels/*.txt`).

use crate::entity::{Entity, Kind};

pub const GRID_SIZE: f64 = 32.0;
pub const LEVEL_COUNT: u32 = 3;

static LEVEL_TXT: [&str; 3] = [
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../assets/levels/level1.txt")),
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../assets/levels/level2.txt")),
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../assets/levels/level3.txt")),
];

/// One digit per tile, one line per row.
pub fn parse_layout(text: &str) -> Vec<Vec<u8>> {
    text.lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.trim().bytes().map(|b| b.wrapping_sub(b'0')).collect())
        .collect()
}

pub fn robot_mistake_chance(level: u32) -> f64 {
    match level.clamp(1, LEVEL_COUNT) {
        1 => 0.25,
        2 => 0.1,
        _ => 0.0,
    }
}

pub fn layout(level: u32) -> Vec<Vec<u8>> {
    parse_layout(LEVEL_TXT[(level.clamp(1, LEVEL_COUNT) - 1) as usize])
}

/// Tile at (col*32 + 16, row*32 + 16), row-major order.
pub fn create_entities(layout: &[Vec<u8>]) -> Vec<Entity> {
    let mut out = Vec::new();
    for (row, line) in layout.iter().enumerate() {
        for (col, &code) in line.iter().enumerate() {
            if let Some(kind) = Kind::from_code(code) {
                out.push(Entity::new(
                    kind,
                    col as f64 * GRID_SIZE + GRID_SIZE / 2.0,
                    row as f64 * GRID_SIZE + GRID_SIZE / 2.0,
                ));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn counts(level: u32) -> [usize; 8] {
        let mut c = [0usize; 8];
        for row in layout(level) {
            for code in row {
                c[code as usize] += 1;
            }
        }
        c
    }

    #[test]
    fn tile_counts_per_level() {
        // index = tile code: 0 empty, 1 block, 2 star, 3 heart, 4 plant, 5 ghost, 6 tetris, 7 win
        assert_eq!(counts(1), [6284, 528, 10, 15, 11, 11, 322, 19]);
        assert_eq!(counts(2), [5626, 528, 10, 15, 13, 14, 375, 19]);
        assert_eq!(counts(3), [5465, 528, 16, 23, 16, 20, 513, 19]);
    }

    #[test]
    fn layout_dimensions() {
        for l in 1..=3 {
            let g = layout(l);
            assert_eq!(g.len(), 25);
            let w = if l == 1 { 288 } else { 264 };
            assert!(g.iter().all(|r| r.len() == w));
        }
    }

    #[test]
    fn win_block_column() {
        for l in 1..=3 {
            let ents = create_entities(&layout(l));
            let wins: Vec<_> = ents.iter().filter(|e| e.kind == Kind::WinBlock).collect();
            assert_eq!(wins.len(), 19);
            assert!(wins.iter().all(|e| e.pos.x == 263.0 * 32.0 + 16.0));
        }
    }

    #[test]
    fn entity_sizes_and_positions() {
        let g = vec![vec![0, 4, 5]];
        let e = create_entities(&g);
        assert_eq!(e.len(), 2);
        assert_eq!((e[0].kind, e[0].pos.x, e[0].pos.y, e[0].size.y), (Kind::Plant, 48.0, 16.0, 64.0));
        assert_eq!((e[1].kind, e[1].size.x), (Kind::Ghost, 50.0));
    }

    #[test]
    fn mistake_chances() {
        assert_eq!(robot_mistake_chance(1), 0.25);
        assert_eq!(robot_mistake_chance(2), 0.1);
        assert_eq!(robot_mistake_chance(3), 0.0);
    }
}
