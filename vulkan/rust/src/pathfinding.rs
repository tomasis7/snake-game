//! Port of ai/pathfinding.ts: pure grid AI for the robot player.

use std::collections::{HashSet, VecDeque};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GridPos {
    pub col: i32,
    pub row: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Dir {
    pub dx: i32,
    pub dy: i32,
}

#[derive(Clone, Copy, Debug)]
pub struct Pickup {
    pub pos: GridPos,
    pub value: f64,
}

pub struct AiWorld {
    pub blocked: HashSet<(i32, i32)>,
    pub min_col: i32,
    pub max_col: i32,
    pub min_row: i32,
    pub max_row: i32,
}

/// Order matters: fallback_dir picks the first free one, so rightward comes first.
const DIRS: [Dir; 4] = [Dir { dx: 1, dy: 0 }, Dir { dx: 0, dy: -1 }, Dir { dx: 0, dy: 1 }, Dir { dx: -1, dy: 0 }];

fn in_bounds(w: &AiWorld, p: GridPos) -> bool {
    p.col >= w.min_col && p.col <= w.max_col && p.row >= w.min_row && p.row <= w.max_row
}

fn is_free(w: &AiWorld, p: GridPos) -> bool {
    in_bounds(w, p) && !w.blocked.contains(&(p.col, p.row))
}

fn is_reverse(a: Dir, b: Dir) -> bool {
    a.dx == -b.dx && a.dy == -b.dy && (a.dx != 0 || a.dy != 0)
}

/// Shortest path via BFS; returns the FIRST step, or None if unreachable.
pub fn bfs_first_step(world: &AiWorld, start: GridPos, target: GridPos, current: Dir) -> Option<Dir> {
    if start == target || !is_free(world, target) {
        return None;
    }
    let mut visited: HashSet<GridPos> = HashSet::new();
    visited.insert(start);
    let mut queue: VecDeque<(GridPos, Dir)> = VecDeque::new();
    for d in DIRS {
        if is_reverse(d, current) {
            continue;
        }
        let next = GridPos { col: start.col + d.dx, row: start.row + d.dy };
        if next == target {
            return Some(d);
        }
        if !is_free(world, next) {
            continue;
        }
        visited.insert(next);
        queue.push_back((next, d));
    }
    while let Some((pos, first)) = queue.pop_front() {
        for d in DIRS {
            let next = GridPos { col: pos.col + d.dx, row: pos.row + d.dy };
            if visited.contains(&next) {
                continue;
            }
            if next == target {
                return Some(first);
            }
            if !is_free(world, next) {
                continue;
            }
            visited.insert(next);
            queue.push_back((next, first));
        }
    }
    None
}

/// Any non-lethal direction, preferring rightward. If all blocked, keep going.
pub fn fallback_dir(world: &AiWorld, start: GridPos, current: Dir) -> Dir {
    for d in DIRS {
        if is_reverse(d, current) {
            continue;
        }
        if is_free(world, GridPos { col: start.col + d.dx, row: start.row + d.dy }) {
            return d;
        }
    }
    current
}

/// Pickups scored by manhattan distance / value (lower = better); those near the
/// rival's head count double; pickups more than 2 columns behind are ignored.
pub fn decide_direction(
    world: &AiWorld,
    start: GridPos,
    current: Dir,
    pickups: &[Pickup],
    rival_head: Option<GridPos>,
) -> Dir {
    let mut scored: Vec<(GridPos, f64)> = pickups
        .iter()
        .filter(|p| p.pos.col >= start.col - 2)
        .map(|p| {
            let mut value = p.value;
            if let Some(r) = rival_head {
                let rd = (p.pos.col - r.col).abs() + (p.pos.row - r.row).abs();
                if rd <= 8 {
                    value *= 2.0;
                }
            }
            let dist = ((p.pos.col - start.col).abs() + (p.pos.row - start.row).abs()) as f64;
            (p.pos, dist / value)
        })
        .collect();
    scored.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    for (pos, _) in scored.iter().take(4) {
        if let Some(step) = bfs_first_step(world, start, *pos, current) {
            return step;
        }
    }
    let right_target = GridPos { col: world.max_col, row: start.row };
    bfs_first_step(world, start, right_target, current).unwrap_or_else(|| fallback_dir(world, start, current))
}

#[cfg(test)]
mod tests {
    use super::*;

    const RIGHT: Dir = Dir { dx: 1, dy: 0 };

    fn gp(col: i32, row: i32) -> GridPos {
        GridPos { col, row }
    }

    fn world(blocked: &[(i32, i32)]) -> AiWorld {
        AiWorld { blocked: blocked.iter().cloned().collect(), min_col: 0, max_col: 20, min_row: 0, max_row: 10 }
    }

    #[test]
    fn bfs_steps_right() {
        let w = world(&[]);
        assert_eq!(bfs_first_step(&w, gp(2, 5), gp(8, 5), RIGHT), Some(Dir { dx: 1, dy: 0 }));
    }

    #[test]
    fn bfs_routes_around_wall() {
        let wall: Vec<(i32, i32)> = (3..=7).map(|r| (4, r)).collect();
        let w = world(&wall);
        let s = bfs_first_step(&w, gp(3, 5), gp(8, 5), RIGHT).unwrap();
        assert!(s.dx == 0 && (s.dy == 1 || s.dy == -1));
    }

    #[test]
    fn bfs_unreachable() {
        let w = world(&[(4, 5), (6, 5), (5, 4), (5, 6)]);
        assert_eq!(bfs_first_step(&w, gp(5, 5), gp(10, 5), RIGHT), None);
    }

    #[test]
    fn bfs_never_reverses() {
        let w = world(&[]);
        assert_ne!(bfs_first_step(&w, gp(5, 5), gp(2, 5), RIGHT), Some(Dir { dx: -1, dy: 0 }));
    }

    #[test]
    fn bfs_blocked_target() {
        let w = world(&[(8, 5)]);
        assert_eq!(bfs_first_step(&w, gp(5, 5), gp(8, 5), RIGHT), None);
    }

    #[test]
    fn fallback_prefers_right() {
        assert_eq!(fallback_dir(&world(&[]), gp(5, 5), RIGHT), Dir { dx: 1, dy: 0 });
    }

    #[test]
    fn fallback_dodges() {
        let d = fallback_dir(&world(&[(6, 5)]), gp(5, 5), RIGHT);
        assert_eq!(d.dx, 0);
        assert_eq!(d.dy.abs(), 1);
    }

    #[test]
    fn fallback_boxed_in() {
        let w = world(&[(4, 5), (6, 5), (5, 4), (5, 6)]);
        assert_eq!(fallback_dir(&w, gp(5, 5), RIGHT), RIGHT);
    }

    #[test]
    fn decide_heads_to_pickup() {
        let d = decide_direction(&world(&[]), gp(2, 5), RIGHT, &[Pickup { pos: gp(2, 2), value: 3.0 }], None);
        assert_eq!(d, Dir { dx: 0, dy: -1 });
    }

    #[test]
    fn decide_prefers_star() {
        let d = decide_direction(
            &world(&[]),
            gp(5, 5),
            RIGHT,
            &[Pickup { pos: gp(5, 9), value: 1.0 }, Pickup { pos: gp(11, 5), value: 3.0 }],
            None,
        );
        assert_eq!(d, Dir { dx: 1, dy: 0 });
    }

    #[test]
    fn decide_contests_rival_pickups() {
        let d = decide_direction(
            &world(&[]),
            gp(10, 5),
            RIGHT,
            &[Pickup { pos: gp(10, 1), value: 1.0 }, Pickup { pos: gp(10, 9), value: 1.0 }],
            Some(gp(10, 10)),
        );
        assert_eq!(d, Dir { dx: 0, dy: 1 });
    }

    #[test]
    fn decide_ignores_pickups_behind() {
        let d = decide_direction(&world(&[]), gp(10, 5), RIGHT, &[Pickup { pos: gp(2, 5), value: 3.0 }], None);
        assert_eq!(d, Dir { dx: 1, dy: 0 });
    }
}
