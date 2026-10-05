//! Parser for the shared `assets/sprites.txt` pixel-art format.

use std::collections::HashMap;

pub static SPRITES_TXT: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../assets/sprites.txt"));

/// Sprite ids, in the order the atlas stores them.
pub const NAMES: [&str; 8] = ["star", "starBright", "heart", "ghost", "plant", "wall", "tetris", "win"];
pub const STAR: usize = 0;
pub const STAR_BRIGHT: usize = 1;
pub const HEART: usize = 2;
pub const GHOST: usize = 3;
pub const PLANT: usize = 4;
pub const WALL: usize = 5;
pub const TETRIS: usize = 6;
pub const WIN: usize = 7;

#[derive(Clone, Debug)]
pub struct Sprite {
    pub name: String,
    pub cols: usize,
    pub rows: Vec<String>,
    pub palette: HashMap<char, u32>,
}

pub fn parse(text: &str) -> Result<Vec<Sprite>, String> {
    let mut out = Vec::new();
    let mut lines = text.lines().map(|l| l.trim_end());
    while let Some(line) = lines.next() {
        if line.is_empty() {
            continue;
        }
        let mut h = line.split_whitespace();
        if h.next() != Some("sprite") {
            return Err(format!("expected 'sprite', got '{line}'"));
        }
        let name = h.next().ok_or("missing name")?.to_string();
        let cols: usize = h.next().and_then(|v| v.parse().ok()).ok_or("bad cols")?;
        let nrows: usize = h.next().and_then(|v| v.parse().ok()).ok_or("bad rows")?;
        let mut palette = HashMap::new();
        loop {
            let l = lines.next().ok_or("unexpected eof in palette")?;
            if l == "rows" {
                break;
            }
            let mut p = l.split_whitespace();
            let ch = p.next().and_then(|c| c.chars().next()).ok_or("bad palette")?;
            let hex = p.next().ok_or("bad palette colour")?.trim_start_matches('#');
            palette.insert(ch, u32::from_str_radix(hex, 16).map_err(|e| e.to_string())?);
        }
        let mut rows = Vec::new();
        for _ in 0..nrows {
            rows.push(lines.next().ok_or("unexpected eof in rows")?.to_string());
        }
        if lines.next() != Some("end") {
            return Err(format!("sprite {name}: missing 'end'"));
        }
        out.push(Sprite { name, cols, rows, palette });
    }
    Ok(out)
}

/// The built-in sprites in `NAMES` order.
pub fn load() -> Vec<Sprite> {
    let all = parse(SPRITES_TXT).expect("sprites.txt");
    NAMES
        .iter()
        .map(|n| all.iter().find(|s| s.name == *n).unwrap_or_else(|| panic!("sprite {n} missing")).clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_every_sprite() {
        let all = parse(SPRITES_TXT).unwrap();
        let mut names: Vec<_> = all.iter().map(|s| s.name.clone()).collect();
        names.sort();
        let mut want: Vec<_> = NAMES.iter().map(|s| s.to_string()).collect();
        want.sort();
        assert_eq!(names, want);
    }

    #[test]
    fn rows_are_rectangular() {
        for s in parse(SPRITES_TXT).unwrap() {
            assert!(!s.rows.is_empty());
            for r in &s.rows {
                assert_eq!(r.chars().count(), s.cols, "{}", s.name);
            }
        }
    }

    #[test]
    fn only_palette_chars() {
        for s in parse(SPRITES_TXT).unwrap() {
            for r in &s.rows {
                for c in r.chars() {
                    assert!(c == '.' || s.palette.contains_key(&c), "{} uses {c}", s.name);
                }
            }
        }
    }

    #[test]
    fn palette_is_hex() {
        for s in parse(SPRITES_TXT).unwrap() {
            for v in s.palette.values() {
                assert!(*v <= 0xffffff);
            }
        }
        assert!(parse("sprite x 1 1\na #zzzzzz\nrows\na\nend\n").is_err());
    }

    #[test]
    fn star_twinkle_shares_grid() {
        let s = load();
        assert_eq!(s[STAR].rows, s[STAR_BRIGHT].rows);
        assert_ne!(s[STAR].palette, s[STAR_BRIGHT].palette);
    }
}
