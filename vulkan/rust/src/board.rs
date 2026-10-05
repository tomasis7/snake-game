//! Port of gameboard.ts: the race screen (update + draw), plus the HUD from racemanager.ts.

use crate::camera::{advance_kill_line, fit_camera, intersects_rect, visible_world_rect, Camera};
use crate::collision::{check_collision, CollisionCtx, CollisionEvent};
use crate::draw::{rgb, rgba, Painter};
use crate::effects::Effects;
use crate::entity::{Entity, Kind};
use crate::input::Input;
use crate::levels;
use crate::player::{Controller, Player, ARROWS, WASD};
use crate::progress::GameMode;
use crate::race::{format_time, RaceManager, RaceReason};
use crate::rng::Rng;
use crate::robot::{Robot, RobotCtx};
use crate::sound::Sound;
use crate::sprites;
use crate::text::HAlign;
use crate::vec2::v2;

/// TS `deltaTime` at frameRate(60).
pub const DT: f64 = 1000.0 / 60.0;

const CANVAS_W: f64 = 1200.0;
const CANVAS_H: f64 = 800.0;
const MAX_GAP: f64 = 1000.0;
const WARN_MARGIN: f64 = 280.0;
const CAMERA_PADDING: f64 = 220.0;
const MIN_SCALE: f64 = 0.5;
const MAX_SCALE: f64 = 1.0;
const BG_TILE_W: f64 = 1415.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RaceEnd {
    pub winner: u32,
    pub reason: RaceReason,
    pub time_ms: f64,
}

/// Everything the board needs from the game for one simulation tick.
pub struct TickCtx<'a> {
    pub clock: f64,
    pub input: &'a Input,
    pub rng: &'a mut Rng,
    pub sounds: &'a mut Vec<Sound>,
}

pub struct GameBoard {
    entities: Vec<Entity>,
    players: Vec<Player>,
    race: RaceManager,
    pub level: u32,
    level_ended: bool,
    effects: Effects,
    kill_line: f64,
    end: Option<RaceEnd>,
}

impl GameBoard {
    /// `both_robots` is the bench/screenshot setup: both snakes on robot AI with mistake chance 0.
    pub fn new(level: u32, mode: GameMode, both_robots: bool) -> GameBoard {
        let chance = levels::robot_mistake_chance(level);
        let p1_ctl = if both_robots { Controller::Robot(Robot::new(0.0)) } else { Controller::Keys(ARROWS) };
        let p2_ctl = if both_robots {
            Controller::Robot(Robot::new(0.0))
        } else if mode == GameMode::OnePlayer {
            Controller::Robot(Robot::new(chance))
        } else {
            Controller::Keys(WASD)
        };
        let p1 = Player::new(v2(128.0, 192.0), 1, 0x00FFFF, 0x008000, p1_ctl);
        let p2 = Player::new(v2(128.0, 576.0), 2, 0xFF00FF, 0xFFA500, p2_ctl);

        let entities = levels::create_entities(&levels::layout(level));
        let finish_x = entities.iter().find(|e| e.kind == Kind::WinBlock).map_or(CANVAS_W * 4.0, |e| e.pos.x);
        let start_x = p1.trail[0].x;
        GameBoard {
            race: RaceManager::new(&[1, 2], start_x, finish_x),
            entities,
            players: vec![p1, p2],
            level,
            level_ended: false,
            effects: Effects::default(),
            kill_line: 0.0,
            end: None,
        }
    }

    fn leader_x(&self) -> f64 {
        self.players.iter().map(|p| p.trail[0].x).fold(f64::NEG_INFINITY, f64::max)
    }

    fn resolve_race(&mut self, winner: u32, reason: RaceReason) {
        if self.level_ended {
            return;
        }
        self.level_ended = true;
        self.race.declare_winner(winner, reason);
        self.end = Some(RaceEnd { winner, reason, time_ms: self.race.elapsed_ms() });
    }

    /// One 60 Hz tick. Returns the race result once the race ends.
    pub fn update(&mut self, c: &mut TickCtx) -> Option<RaceEnd> {
        if self.level_ended {
            return None;
        }
        self.update_inner(c);
        self.end
    }

    fn update_inner(&mut self, c: &mut TickCtx) {
        // The kill line trails the leader and only advances.
        self.kill_line = advance_kill_line(self.kill_line, self.leader_x(), MAX_GAP);

        for i in 0..self.players.len() {
            if self.players[i].is_robot() {
                let other = self.players[1 - i].trail.clone();
                let ctx = RobotCtx { entities: &self.entities, camera_offset: self.kill_line, other_trail: &other };
                self.players[i].update(DT, c.clock, c.input, Some(&ctx), c.rng);
            } else {
                self.players[i].update(DT, c.clock, c.input, None, c.rng);
            }
        }

        for i in 0..self.players.len() {
            let p = &self.players[i];
            if p.trail[0].x + p.size.x < self.kill_line {
                let other = if p.number == 1 { 2 } else { 1 };
                self.resolve_race(other, RaceReason::FellBehind);
                if self.level_ended {
                    return;
                }
            }
        }

        for e in &mut self.entities {
            e.update(c.clock);
        }
        // flyingGhost(): ghosts are updated a second time per frame.
        for e in &mut self.entities {
            if e.kind == Kind::Ghost {
                e.update(c.clock);
            }
        }

        let mut events = Vec::new();
        {
            let mut cc = CollisionCtx {
                fx: &mut self.effects,
                rng: c.rng,
                sounds: c.sounds,
                events: &mut events,
                clock: c.clock,
            };
            check_collision(&mut self.players, &mut self.entities, &mut cc);
        }
        for ev in events {
            match ev {
                CollisionEvent::Finish(pn) => self.resolve_race(pn, RaceReason::Finish),
                CollisionEvent::Eliminate(pn) => {
                    self.resolve_race(if pn == 1 { 2 } else { 1 }, RaceReason::OpponentOut)
                }
            }
        }

        for p in &self.players {
            self.race.set_head_x(p.number, p.trail[0].x);
        }
        self.race.tick(DT);
        self.effects.update(DT);
    }

    fn camera(&self, acc: f64) -> Camera {
        let heads: Vec<_> = self.players.iter().map(|p| p.interpolated_head(acc)).collect();
        let xs: Vec<f64> = heads.iter().map(|h| h.x).collect();
        let ys: Vec<f64> = heads.iter().map(|h| h.y).collect();
        fit_camera(&xs, &ys, CANVAS_W, CANVAS_H, CAMERA_PADDING, MIN_SCALE, MAX_SCALE)
    }

    /// `clock` is the game clock (TS `millis()`), `acc` the fixed-step leftover in ms.
    pub fn draw(&self, p: &mut Painter, clock: f64, acc: f64, rng: &mut Rng) {
        let cam = self.camera(acc);
        let (shake_x, shake_y) = self.effects.shake_offset(rng);
        let sc = cam.scale;
        let tx = |wx: f64| (wx - cam.center_x) * sc + CANVAS_W / 2.0 + shake_x;
        let ty = |wy: f64| (wy - cam.center_y) * sc + CANVAS_H / 2.0 + shake_y;

        // Screen-space background with parallax from the camera centre.
        let bg_shift = ((cam.center_x * 0.25) % BG_TILE_W + BG_TILE_W) % BG_TILE_W;
        let tiles = (CANVAS_W / BG_TILE_W).ceil() as i32 + 2;
        for i in 0..tiles {
            p.background(i as f64 * BG_TILE_W - bg_shift + shake_x, shake_y, BG_TILE_W, CANVAS_H);
        }

        // Entities, culled to the visible world rect.
        let view = visible_world_rect(&cam, CANVAS_W, CANVAS_H, 64.0);
        for e in &self.entities {
            if e.removed || !intersects_rect(e.pos.x, e.pos.y, e.size.x, e.size.y, &view) {
                continue;
            }
            let (mut y, mut w, mut h) = (e.pos.y, e.size.x, e.size.y);
            let id = match e.kind {
                Kind::Block => sprites::WALL,
                Kind::Tetris => sprites::TETRIS,
                Kind::WinBlock => sprites::WIN,
                Kind::Plant => sprites::PLANT,
                Kind::Star => {
                    if clock % 800.0 < 400.0 {
                        sprites::STAR
                    } else {
                        sprites::STAR_BRIGHT
                    }
                }
                Kind::Heart => {
                    w *= e.pulse_scale;
                    h *= e.pulse_scale;
                    sprites::HEART
                }
                Kind::Ghost => {
                    y += (clock / 400.0).sin() * 3.0;
                    sprites::GHOST
                }
            };
            p.sprite_center(id, tx(e.pos.x), ty(y), w * sc, h * sc);
        }

        for pl in &self.players {
            self.draw_player(p, pl, clock, acc, &tx, &ty, sc);
        }

        // Effects, world space.
        for pt in &self.effects.particles {
            let fade = (pt.life / pt.max_life) as f32;
            p.solid_center(tx(pt.x), ty(pt.y), pt.size * sc, pt.size * sc, rgba(pt.color, fade));
        }
        for t in &self.effects.texts {
            let fade = (t.life / t.max_life) as f32;
            p.text(&t.text, tx(t.x), ty(t.y), 16.0 * sc, rgba(t.color, fade), HAlign::Center);
        }

        // Flash overlay, screen space.
        let alpha = self.effects.flash_alpha();
        if alpha > 0.0 {
            p.solid(0.0, 0.0, CANVAS_W, CANVAS_H, rgba(self.effects.flash_color, (alpha / 255.0) as f32));
        }

        self.draw_warnings(p, &cam, clock);
        self.draw_hud(p);
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_player(
        &self,
        p: &mut Painter,
        pl: &Player,
        clock: f64,
        acc: f64,
        tx: &dyn Fn(f64) -> f64,
        ty: &dyn Fn(f64) -> f64,
        sc: f64,
    ) {
        if pl.is_blinked_out(clock) {
            return;
        }
        let d = pl.size.x.max(pl.size.y) * sc;
        let head_mid = rgb(0xFFA500);
        let head_edge = rgb(0x804600);
        let body_mid = rgb(pl.fill);
        let body_edge = {
            let s = rgb(pl.stroke);
            [s[0] * 0.3, s[1] * 0.3, s[2] * 0.3, 1.0]
        };
        let shadow = rgba(0x000000, 0.3);
        for i in 0..pl.trail.len() {
            let wp = pl.render_pos(i, acc);
            let (cx, cy) = (tx(wp.x), ty(wp.y));
            // Canvas shadow: offset +5,+5 in screen pixels, blur 15 -> about 1.6x the diameter.
            p.shadow(cx + 5.0, cy + 5.0, d * 1.6, shadow);
            if i == 0 {
                p.ball(cx, cy, d, head_mid, head_edge, 0.8);
                self.draw_eyes(p, pl, wp.x, wp.y, tx, ty, sc);
            } else {
                p.ball(cx, cy, d, body_mid, body_edge, 0.2);
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_eyes(&self, p: &mut Painter, pl: &Player, hx: f64, hy: f64, tx: &dyn Fn(f64) -> f64, ty: &dyn Fn(f64) -> f64, sc: f64) {
        let sign = |v: f64| if v > 0.0 { 1.0 } else if v < 0.0 { -1.0 } else { 0.0 };
        let (fx, fy) = (sign(pl.direction.x), sign(pl.direction.y));
        let (sx, sy) = (fy, fx);
        let diameter = pl.size.x.max(pl.size.y);
        let eye = diameter * 0.13;
        let fwd = diameter * 0.18;
        let side = diameter * 0.18;
        for s in [1.0, -1.0] {
            let x = hx + fx * fwd + sx * side * s;
            let y = hy + fy * fwd + sy * side * s;
            p.solid_center(tx(x), ty(y), eye * 2.0 * sc, eye * 2.0 * sc, rgb(0xffffff));
            p.solid_center(tx(x + fx * eye * 0.4), ty(y + fy * eye * 0.4), eye * sc, eye * sc, rgb(0x101820));
        }
    }

    /// Flashing warning over any racer near the kill line.
    fn draw_warnings(&self, p: &mut Painter, cam: &Camera, clock: f64) {
        if clock % 500.0 >= 300.0 {
            return;
        }
        for pl in &self.players {
            let head = pl.trail[0];
            if head.x >= self.kill_line + WARN_MARGIN {
                continue;
            }
            let sx = (head.x - cam.center_x) * cam.scale + CANVAS_W / 2.0;
            let sy = (head.y - cam.center_y) * cam.scale + CANVAS_H / 2.0;
            p.text("OUT OF TIME!", sx, sy - 40.0, 18.0, rgb(0xff2d55), HAlign::Center);
        }
    }

    /// Progress bar with markers, finish flag, clock and lives pips.
    fn draw_hud(&self, p: &mut Painter) {
        let bar_x = 210.0;
        let bar_w = CANVAS_W - bar_x - 60.0;
        let bar_y = 30.0;
        let racer_color = |pn: u32| if pn == 1 { 0x00FFFF } else { 0xFF00FF };

        p.text(&format!("LVL {}/{}", self.level, levels::LEVEL_COUNT), 20.0, bar_y, 14.0, rgb(0x45FF8C), HAlign::Left);
        p.rounded(bar_x, bar_y - 6.0, bar_w, 12.0, 6.0, rgb(0x2a2a2a));
        p.solid(bar_x + bar_w, bar_y - 10.0, 6.0, 20.0, rgb(0x45FF8C));
        for pl in &self.players {
            let x = bar_x + bar_w * self.race.progress(pl.number);
            p.rounded_center(x, bar_y, 12.0, 18.0, 3.0, rgb(racer_color(pl.number)));
        }
        p.text(&format_time(self.race.elapsed_ms()), CANVAS_W - 20.0, bar_y, 14.0, rgb(0xffffff), HAlign::Right);

        let mut ly = bar_y + 18.0;
        for pl in &self.players {
            for i in 0..pl.lives.max(0) {
                p.rounded(20.0 + i as f64 * 14.0, ly, 10.0, 10.0, 2.0, rgb(racer_color(pl.number)));
            }
            ly += 16.0;
        }
    }
}
