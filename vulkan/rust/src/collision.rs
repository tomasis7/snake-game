//! Port of collisionmanager.ts. The head's `trail[0]` and every entity `pos` are
//! treated as top-left corners for collision, exactly as in the TS.

use crate::effects::Effects;
use crate::entity::{Entity, Kind};
use crate::player::Player;
use crate::rng::Rng;
use crate::sound::Sound;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CollisionEvent {
    Finish(u32),
    Eliminate(u32),
}

pub struct CollisionCtx<'a> {
    pub fx: &'a mut Effects,
    pub rng: &'a mut Rng,
    pub sounds: &'a mut Vec<Sound>,
    pub events: &'a mut Vec<CollisionEvent>,
    pub clock: f64,
}

/// A hazard costs one life and stuns, throttled by the collision cooldown.
fn handle_hazard(player: &mut Player, c: &mut CollisionCtx) {
    if c.clock - player.last_collision_time < player.collision_cooldown {
        return;
    }
    player.last_collision_time = c.clock;
    if player.can_pass_through_obstacles {
        return;
    }
    c.sounds.push(Sound::BlockCollision);
    c.fx.shake(10.0);
    c.fx.flash(0xff2d55);
    let head = player.trail[0];
    c.fx.burst(c.rng, head.x, head.y, 0xff2d55, 10);
    player.apply_stun(c.clock, 600.0);
    player.is_colliding = true;
    player.lives -= 1;
    if player.lives <= 0 {
        player.lives = 0;
        c.sounds.push(Sound::MusicStop);
        c.events.push(CollisionEvent::Eliminate(player.number));
    }
}

fn handle_finish(player: &Player, c: &mut CollisionCtx) {
    c.sounds.push(Sound::Goalline);
    c.fx.flash(0x45ff8c);
    c.events.push(CollisionEvent::Finish(player.number));
}

fn handle_star(star: &mut Entity, c: &mut CollisionCtx) {
    if star.removed {
        return;
    }
    c.sounds.push(Sound::StarPickUp);
    c.fx.burst(c.rng, star.pos.x, star.pos.y, 0xffd93b, 14);
    star.removed = true;
}

fn handle_heart(player: &mut Player, heart: &mut Entity, c: &mut CollisionCtx) {
    if heart.removed {
        return;
    }
    c.sounds.push(Sound::GainHeart);
    c.fx.burst(c.rng, heart.pos.x, heart.pos.y, 0xe8384f, 14);
    if player.lives < player.max_lives {
        player.lives += 1;
    }
    heart.removed = true;
}

fn handle_ghost_proximity(player: &Player, ghost: &mut Entity, sounds: &mut Vec<Sound>) {
    let head = player.trail[0];
    let d = ((head.x - ghost.pos.x).powi(2) + (head.y - ghost.pos.y).powi(2)).sqrt();
    if d < 200.0 {
        if !ghost.sound_playing {
            sounds.push(Sound::GhostPlay);
            ghost.sound_playing = true;
        } else {
            sounds.push(Sound::GhostStop);
            ghost.sound_playing = false;
        }
    }
}

/// Removed entities stay in the list and still overlap (as in the TS, where the
/// collision manager keeps its original array); their handlers just do nothing.
pub fn check_collision(players: &mut [Player], entities: &mut [Entity], c: &mut CollisionCtx) {
    for player in players.iter_mut() {
        let head = player.trail[0];
        let (hl, hr) = (head.x, head.x + player.size.x);
        let (ht, hb) = (head.y, head.y + player.size.y);
        let mut has_collision = false;

        for entity in entities.iter_mut() {
            if entity.kind == Kind::Ghost {
                handle_ghost_proximity(player, entity, c.sounds);
            }
            let colliding = hr > entity.pos.x
                && hl < entity.pos.x + entity.size.x
                && hb > entity.pos.y
                && ht < entity.pos.y + entity.size.y;

            if colliding {
                has_collision = true;
                if !player.is_colliding {
                    match entity.kind {
                        k if k.is_hazard() => handle_hazard(player, c),
                        Kind::Star => handle_star(entity, c),
                        Kind::Heart => handle_heart(player, entity, c),
                        Kind::WinBlock => handle_finish(player, c),
                        _ => {}
                    }
                    break;
                }
            }
            if !has_collision {
                player.is_colliding = false;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::player::{Controller, ARROWS};
    use crate::vec2::v2;

    struct Rig {
        fx: Effects,
        rng: Rng,
        sounds: Vec<Sound>,
        events: Vec<CollisionEvent>,
    }

    impl Rig {
        fn new() -> Rig {
            Rig { fx: Effects::default(), rng: Rng::new(42), sounds: vec![], events: vec![] }
        }
        fn run(&mut self, p: &mut Player, e: &mut [Entity], clock: f64) {
            let mut c = CollisionCtx {
                fx: &mut self.fx,
                rng: &mut self.rng,
                sounds: &mut self.sounds,
                events: &mut self.events,
                clock,
            };
            check_collision(std::slice::from_mut(p), e, &mut c);
        }
    }

    fn player() -> Player {
        Player::new(v2(128.0, 192.0), 1, 0x00ffff, 0x008000, Controller::Keys(ARROWS))
    }

    /// An entity whose top-left is the player's head.
    fn on_head(p: &Player, kind: Kind) -> Entity {
        Entity::new(kind, p.trail[0].x + 1.0, p.trail[0].y + 1.0)
    }

    #[test]
    fn hazard_costs_life_cooldown_and_stun() {
        let mut p = player();
        let mut rig = Rig::new();
        let mut e = [on_head(&p, Kind::Block)];
        rig.run(&mut p, &mut e, 5000.0);
        assert_eq!(p.lives, 2);
        assert_eq!(p.last_collision_time, 5000.0);
        assert!(p.is_stunned(5100.0));
        assert!(!p.is_stunned(5700.0));
        assert!(rig.sounds.contains(&Sound::BlockCollision));
        assert!(rig.fx.particle_count() == 10 && rig.fx.flash_alpha() > 0.0);
        // Still overlapping: no second hit while isColliding.
        rig.run(&mut p, &mut e, 5016.0);
        assert_eq!(p.lives, 2);
    }

    #[test]
    fn hazard_respects_cooldown() {
        let mut p = player();
        let mut rig = Rig::new();
        let mut e = [on_head(&p, Kind::Plant)];
        rig.run(&mut p, &mut e, 5000.0);
        p.is_colliding = false;
        rig.run(&mut p, &mut e, 5500.0);
        assert_eq!(p.lives, 2);
        p.is_colliding = false;
        rig.run(&mut p, &mut e, 6100.0);
        assert_eq!(p.lives, 1);
    }

    #[test]
    fn last_life_eliminates() {
        let mut p = player();
        p.lives = 1;
        let mut rig = Rig::new();
        let mut e = [on_head(&p, Kind::Tetris)];
        rig.run(&mut p, &mut e, 5000.0);
        assert_eq!(p.lives, 0);
        assert_eq!(rig.events, vec![CollisionEvent::Eliminate(1)]);
    }

    #[test]
    fn heart_adds_life_capped_at_ten() {
        let mut p = player();
        let mut rig = Rig::new();
        let mut e = [on_head(&p, Kind::Heart)];
        rig.run(&mut p, &mut e, 5000.0);
        assert_eq!(p.lives, 4);
        assert!(e[0].removed);
        let mut p = player();
        p.lives = 10;
        let mut e = [on_head(&p, Kind::Heart)];
        rig.run(&mut p, &mut e, 5000.0);
        assert_eq!(p.lives, 10);
    }

    #[test]
    fn win_block_finishes_race() {
        let mut p = player();
        let mut rig = Rig::new();
        let mut e = [on_head(&p, Kind::WinBlock)];
        rig.run(&mut p, &mut e, 5000.0);
        assert_eq!(rig.events, vec![CollisionEvent::Finish(1)]);
        assert!(rig.sounds.contains(&Sound::Goalline));
    }

    #[test]
    fn star_is_collected_once() {
        let mut p = player();
        let mut rig = Rig::new();
        let mut e = [on_head(&p, Kind::Star)];
        rig.run(&mut p, &mut e, 5000.0);
        assert!(e[0].removed);
        let n = rig.fx.particle_count();
        rig.run(&mut p, &mut e, 5016.0);
        assert_eq!(rig.fx.particle_count(), n);
    }

    #[test]
    fn no_collision_when_apart() {
        let mut p = player();
        let mut rig = Rig::new();
        let mut e = [Entity::new(Kind::Block, p.trail[0].x + 200.0, p.trail[0].y)];
        rig.run(&mut p, &mut e, 5000.0);
        assert_eq!(p.lives, 3);
    }
}
