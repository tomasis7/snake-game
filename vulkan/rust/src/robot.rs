//! Port of robotplayer.ts: the robot's decision step (the Player owns the rest).

use std::collections::HashSet;

use crate::entity::{Entity, Kind};
use crate::pathfinding::{decide_direction, fallback_dir, AiWorld, Dir, GridPos, Pickup};
use crate::rng::Rng;
use crate::vec2::{v2, V2};

const CELL: f64 = 32.0;
const CANVAS_W: f64 = 1200.0;
const CANVAS_H: f64 = 800.0;

pub struct Robot {
    pub mistake_chance: f64,
    think_timer: f64,
}

impl Robot {
    pub fn new(mistake_chance: f64) -> Robot {
        Robot { mistake_chance, think_timer: 0.0 }
    }
}

pub struct RobotCtx<'a> {
    pub entities: &'a [Entity],
    pub camera_offset: f64,
    /// The rival's trail (TS `otherTrails[0]`).
    pub other_trail: &'a [V2],
}

fn to_grid(x: f64, y: f64) -> GridPos {
    GridPos { col: (x / CELL).floor() as i32, row: (y / CELL).floor() as i32 }
}

fn sign(v: f64) -> i32 {
    if v > 0.0 {
        1
    } else if v < 0.0 {
        -1
    } else {
        0
    }
}

fn build_world(own_trail: &[V2], ctx: &RobotCtx) -> (AiWorld, Vec<Pickup>) {
    let min_col = ((ctx.camera_offset / CELL).floor() as i32).max(0);
    let max_col = min_col + (CANVAS_W / CELL).ceil() as i32 + 4;
    let mut world = AiWorld {
        blocked: HashSet::new(),
        min_col,
        max_col,
        min_row: 0,
        max_row: (CANVAS_H / CELL).floor() as i32 - 1,
    };
    let mut pickups = Vec::new();
    for e in ctx.entities {
        if e.removed {
            continue;
        }
        let g = to_grid(e.pos.x, e.pos.y);
        if g.col < min_col - 2 || g.col > max_col {
            continue;
        }
        match e.kind {
            Kind::Star => pickups.push(Pickup { pos: g, value: 3.0 }),
            Kind::Heart => pickups.push(Pickup { pos: g, value: 1.0 }),
            Kind::WinBlock => {}
            _ => {
                world.blocked.insert((g.col, g.row));
            }
        }
    }
    for s in ctx.other_trail {
        let g = to_grid(s.x, s.y);
        world.blocked.insert((g.col, g.row));
    }
    for s in own_trail.iter().skip(1) {
        let g = to_grid(s.x, s.y);
        world.blocked.insert((g.col, g.row));
    }
    (world, pickups)
}

/// Runs the robot's `handleInput`. Returns a new `nextDirection` when it decides one.
pub fn think(robot: &mut Robot, trail: &[V2], direction: V2, ctx: &RobotCtx, dt: f64, rng: &mut Rng) -> Option<V2> {
    robot.think_timer += dt;
    if robot.think_timer < 200.0 {
        return None;
    }
    robot.think_timer = 0.0;

    let (world, pickups) = build_world(trail, ctx);
    let head = to_grid(trail[0].x, trail[0].y);
    let current = Dir { dx: sign(direction.x), dy: sign(direction.y) };

    // Difficulty knob: on a "mistake" tick the robot stops chasing pickups,
    // but still refuses lethal moves.
    if rng.random() < robot.mistake_chance {
        let safe = fallback_dir(&world, head, current);
        return Some(v2(safe.dx as f64 * CELL, safe.dy as f64 * CELL));
    }

    // Contested targeting is the perfect-robot (level 3) behaviour only.
    let rival_head = if robot.mistake_chance == 0.0 && !ctx.other_trail.is_empty() {
        Some(to_grid(ctx.other_trail[0].x, ctx.other_trail[0].y))
    } else {
        None
    };

    // Survival mode: close to the kill line, stop chasing pickups and push right.
    let chasing: &[Pickup] = if trail[0].x - ctx.camera_offset >= 200.0 { &pickups } else { &[] };

    let next = decide_direction(&world, head, current, chasing, rival_head);
    Some(v2(next.dx as f64 * CELL, next.dy as f64 * CELL))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn robot_pushes_right_in_open_field() {
        let mut r = Robot::new(0.0);
        let trail: Vec<V2> = (0..8).map(|i| v2(500.0 - 32.0 * i as f64, 208.0)).collect();
        let ctx = RobotCtx { entities: &[], camera_offset: 0.0, other_trail: &[] };
        let mut rng = Rng::new(42);
        assert_eq!(think(&mut r, &trail, v2(32.0, 0.0), &ctx, 250.0, &mut rng), Some(v2(32.0, 0.0)));
        // Thinks only every 200 ms.
        assert_eq!(think(&mut r, &trail, v2(32.0, 0.0), &ctx, 16.0, &mut rng), None);
    }

    #[test]
    fn robot_dodges_a_block() {
        let mut r = Robot::new(0.0);
        let trail: Vec<V2> = (0..8).map(|i| v2(500.0 - 32.0 * i as f64, 208.0)).collect();
        let g = to_grid(500.0, 208.0);
        let blocks = [Entity::new(Kind::Block, (g.col + 1) as f64 * 32.0 + 16.0, g.row as f64 * 32.0 + 16.0)];
        let ctx = RobotCtx { entities: &blocks, camera_offset: 0.0, other_trail: &[] };
        let d = think(&mut r, &trail, v2(32.0, 0.0), &ctx, 250.0, &mut Rng::new(1)).unwrap();
        assert_eq!(d.x, 0.0);
        assert!(d.y != 0.0);
    }
}
