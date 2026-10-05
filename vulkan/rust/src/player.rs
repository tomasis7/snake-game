//! Port of player.ts (logic; drawing is in board.rs).

use crate::input::{Input, Key};
use crate::rng::Rng;
use crate::robot::{self, Robot, RobotCtx};
use crate::vec2::{v2, V2};

#[derive(Clone, Copy)]
pub struct KeyBindings {
    pub up: Key,
    pub down: Key,
    pub right: Key,
    pub left: Key,
}

pub const ARROWS: KeyBindings = KeyBindings { up: Key::Up, down: Key::Down, right: Key::Right, left: Key::Left };
pub const WASD: KeyBindings = KeyBindings { up: Key::W, down: Key::S, right: Key::D, left: Key::A };

pub enum Controller {
    Keys(KeyBindings),
    Robot(Robot),
}

pub struct Player {
    pub trail: Vec<V2>,
    /// Grid positions before the last step; the drawn snake slides from these to `trail`.
    prev_trail: Vec<V2>,
    pub number: u32,
    pub fill: u32,
    pub stroke: u32,
    move_timer: f64,
    stunned_until: f64,
    pub direction: V2,
    pub next_direction: V2,
    pub controller: Controller,
    pub size: V2,
    pub lives: i32,
    pub max_lives: i32,
    pub last_collision_time: f64,
    pub collision_cooldown: f64,
    pub can_pass_through_obstacles: bool,
    pub is_colliding: bool,
}

impl Player {
    /// `start` is the TS constructor position (before the +16 centring).
    pub fn new(start: V2, number: u32, fill: u32, stroke: u32, controller: Controller) -> Player {
        let size = v2(32.0, 32.0);
        let pos = v2(start.x + 16.0, start.y + 16.0);
        let trail: Vec<V2> = (1..=8).map(|i| v2(pos.x - size.x * i as f64, pos.y)).collect();
        Player {
            prev_trail: trail.clone(),
            trail,
            number,
            fill,
            stroke,
            move_timer: 0.0,
            stunned_until: 0.0,
            direction: v2(32.0, 0.0),
            next_direction: v2(32.0, 0.0),
            controller,
            size,
            lives: 3,
            max_lives: 10,
            // Far in the past, so the hit blink does not show at game-clock zero.
            last_collision_time: -1.0e12,
            collision_cooldown: 1000.0,
            can_pass_through_obstacles: false,
            is_colliding: false,
        }
    }

    pub fn is_robot(&self) -> bool {
        matches!(self.controller, Controller::Robot(_))
    }

    pub fn apply_stun(&mut self, clock: f64, duration_ms: f64) {
        self.stunned_until = clock + duration_ms;
    }

    #[cfg(test)]
    pub fn is_stunned(&self, clock: f64) -> bool {
        clock < self.stunned_until
    }

    fn handle_input(&mut self, dt: f64, input: &Input, ctx: Option<&RobotCtx>, rng: &mut Rng) {
        match &mut self.controller {
            Controller::Keys(k) => {
                if input.key_down(k.up) && self.direction.y == 0.0 {
                    self.next_direction = v2(0.0, -32.0);
                } else if input.key_down(k.down) && self.direction.y == 0.0 {
                    self.next_direction = v2(0.0, 32.0);
                } else if input.key_down(k.left) && self.direction.x == 0.0 {
                    self.next_direction = v2(-32.0, 0.0);
                } else if input.key_down(k.right) && self.direction.x == 0.0 {
                    self.next_direction = v2(32.0, 0.0);
                }
            }
            Controller::Robot(r) => {
                if let Some(ctx) = ctx {
                    if let Some(d) = robot::think(r, &self.trail, self.direction, ctx, dt, rng) {
                        self.next_direction = d;
                    }
                }
            }
        }
    }

    /// One 60 Hz simulation tick (`dt` = TS deltaTime, `clock` = game clock in ms).
    pub fn update(&mut self, dt: f64, clock: f64, input: &Input, ctx: Option<&RobotCtx>, rng: &mut Rng) {
        if clock < self.stunned_until {
            return;
        }
        self.move_timer += dt;
        if self.move_timer >= 200.0 {
            self.move_timer = -100.0;
            self.prev_trail = self.trail.clone();
            self.direction = self.next_direction;
            let head = self.trail[0];
            let new_head = v2(head.x + self.direction.x, head.y + self.direction.y);
            self.trail.insert(0, new_head);
            self.trail.pop();
        }
        self.handle_input(dt, input, ctx, rng);
    }

    /// How far through the current step: the move timer runs -100..200, a 300 ms span.
    /// `acc_ms` is the render-time leftover of the fixed-step accumulator.
    pub fn move_progress(&self, acc_ms: f64) -> f64 {
        ((self.move_timer + acc_ms + 100.0) / 300.0).clamp(0.0, 1.0)
    }

    pub fn render_pos(&self, i: usize, acc_ms: f64) -> V2 {
        let t = self.move_progress(acc_ms);
        let cur = self.trail[i];
        let prev = self.prev_trail.get(i).copied().unwrap_or(cur);
        v2(prev.x + (cur.x - prev.x) * t, prev.y + (cur.y - prev.y) * t)
    }

    pub fn interpolated_head(&self, acc_ms: f64) -> V2 {
        self.render_pos(0, acc_ms)
    }

    /// Hit blink: hidden on even 80 ms phases during the post-hit cooldown.
    pub fn is_blinked_out(&self, clock: f64) -> bool {
        let since = clock - self.last_collision_time;
        since < self.collision_cooldown && (since / 80.0).floor() as i64 % 2 == 0
    }

    #[allow(dead_code)] // part of the TS API
    pub fn double_lives(&mut self) {
        self.lives = (self.lives * 2).min(self.max_lives);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DT: f64 = 1000.0 / 60.0;

    fn human() -> Player {
        Player::new(v2(128.0, 192.0), 1, 0x00ffff, 0x008000, Controller::Keys(ARROWS))
    }

    fn run(p: &mut Player, input: &Input, ticks: usize, clock: &mut f64) {
        let mut rng = Rng::new(1);
        for _ in 0..ticks {
            p.update(DT, *clock, input, None, &mut rng);
            *clock += DT;
        }
    }

    #[test]
    fn starts_with_eight_segments_left_of_position() {
        let p = human();
        assert_eq!(p.trail.len(), 8);
        assert_eq!(p.trail[0], v2(112.0, 208.0));
        assert_eq!(p.trail[7], v2(144.0 - 256.0, 208.0));
        assert_eq!(p.lives, 3);
    }

    /// Ticks until the head moves (at most `max` ticks); returns how many ticks it took.
    fn ticks_to_step(p: &mut Player, input: &Input, clock: &mut f64, max: usize) -> usize {
        let before = p.trail[0];
        let mut rng = Rng::new(1);
        for n in 1..=max {
            p.update(DT, *clock, input, None, &mut rng);
            *clock += DT;
            if p.trail[0] != before {
                return n;
            }
        }
        panic!("no step within {max} ticks");
    }

    #[test]
    fn steps_one_cell_after_200ms_then_every_300() {
        let mut p = human();
        let inp = Input::default();
        let mut clock = 0.0;
        let first = ticks_to_step(&mut p, &inp, &mut clock, 20);
        assert!((12..=13).contains(&first), "first step after {first} ticks");
        assert_eq!(p.trail[0], v2(144.0, 208.0));
        assert_eq!(p.trail.len(), 8);
        let second = ticks_to_step(&mut p, &inp, &mut clock, 30);
        assert!((18..=19).contains(&second), "second step after {second} ticks");
        assert_eq!(p.trail[0].x, 176.0);
    }

    #[test]
    fn cannot_reverse_into_itself() {
        let mut p = human();
        let mut inp = Input::default();
        inp.set_key(Key::Left, true);
        let mut clock = 0.0;
        run(&mut p, &inp, 12 + 18 * 2, &mut clock);
        assert!(p.trail[0].x > 112.0, "head went {:?}", p.trail[0]);
        assert_eq!(p.trail[0].y, 208.0);
    }

    #[test]
    fn turns_up_and_then_cannot_reverse_down() {
        let mut p = human();
        let mut inp = Input::default();
        inp.set_key(Key::Up, true);
        let mut clock = 0.0;
        ticks_to_step(&mut p, &inp, &mut clock, 20); // Up is polled every tick, so the first step goes up
        assert_eq!(p.trail[0], v2(112.0, 176.0));
        inp.set_key(Key::Up, false);
        inp.set_key(Key::Down, true);
        ticks_to_step(&mut p, &inp, &mut clock, 30); // still up: Down would reverse
        ticks_to_step(&mut p, &inp, &mut clock, 30);
        assert!(p.trail[0].y < 176.0);
    }

    #[test]
    fn stun_freezes_movement() {
        let mut p = human();
        let inp = Input::default();
        p.apply_stun(0.0, 600.0);
        let mut clock = 0.0;
        run(&mut p, &inp, 30, &mut clock); // 500 ms
        assert_eq!(p.trail[0].x, 112.0);
    }

    #[test]
    fn interpolation_progress() {
        let p = human();
        assert!((p.move_progress(0.0) - 1.0 / 3.0).abs() < 1e-9);
        assert_eq!(p.move_progress(1000.0), 1.0);
    }
}
