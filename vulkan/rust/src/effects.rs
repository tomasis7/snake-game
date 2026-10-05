//! Port of effects/effects.ts: particles, shake, flash and floating text (logic only).

use crate::rng::Rng;

pub struct Particle {
    pub x: f64,
    pub y: f64,
    vx: f64,
    vy: f64,
    pub life: f64,
    pub max_life: f64,
    pub color: u32,
    pub size: f64,
}

pub struct FloatingText {
    pub x: f64,
    pub y: f64,
    pub text: String,
    pub color: u32,
    pub life: f64,
    pub max_life: f64,
}

const PARTICLE_LIFE_MS: f64 = 600.0;
const TEXT_LIFE_MS: f64 = 1200.0;
const SHAKE_DURATION_MS: f64 = 350.0;
const FLASH_DURATION_MS: f64 = 250.0;
const GRAVITY: f64 = 0.0006;
const MAX_PARTICLES: usize = 240;
const MAX_TEXTS: usize = 12;
const MAX_STEP_MS: f64 = 100.0;

pub struct Effects {
    pub particles: Vec<Particle>,
    pub texts: Vec<FloatingText>,
    shake_intensity: f64,
    shake_remaining: f64,
    flash_remaining: f64,
    pub flash_color: u32,
}

impl Default for Effects {
    fn default() -> Self {
        Effects {
            particles: Vec::new(),
            texts: Vec::new(),
            shake_intensity: 0.0,
            shake_remaining: 0.0,
            flash_remaining: 0.0,
            flash_color: 0xffffff,
        }
    }
}

impl Effects {
    #[cfg(test)]
    pub fn particle_count(&self) -> usize {
        self.particles.len()
    }

    #[cfg(test)]
    pub fn text_count(&self) -> usize {
        self.texts.len()
    }

    /// Flash overlay alpha on the 0..255 scale.
    pub fn flash_alpha(&self) -> f64 {
        if self.flash_remaining <= 0.0 {
            0.0
        } else {
            self.flash_remaining / FLASH_DURATION_MS * 70.0
        }
    }

    pub fn burst(&mut self, rng: &mut Rng, x: f64, y: f64, color: u32, count: usize) {
        for i in 0..count {
            let angle = std::f64::consts::PI * 2.0 * i as f64 / count as f64 + rng.random() * 0.4;
            let speed = 0.06 + rng.random() * 0.12;
            self.particles.push(Particle {
                x,
                y,
                vx: angle.cos() * speed,
                vy: angle.sin() * speed,
                life: PARTICLE_LIFE_MS,
                max_life: PARTICLE_LIFE_MS,
                color,
                size: 3.0 + rng.random() * 3.0,
            });
        }
        if self.particles.len() > MAX_PARTICLES {
            let extra = self.particles.len() - MAX_PARTICLES;
            self.particles.drain(0..extra);
        }
    }

    pub fn shake(&mut self, intensity: f64) {
        self.shake_intensity = self.shake_intensity.max(intensity);
        self.shake_remaining = self.shake_remaining.max(SHAKE_DURATION_MS);
    }

    pub fn flash(&mut self, color: u32) {
        self.flash_color = color;
        self.flash_remaining = FLASH_DURATION_MS;
    }

    #[allow(dead_code)] // part of the TS API; the game never floats text
    pub fn float_text(&mut self, x: f64, y: f64, text: &str, color: u32) {
        self.texts.push(FloatingText {
            x,
            y,
            text: text.to_string(),
            color,
            life: TEXT_LIFE_MS,
            max_life: TEXT_LIFE_MS,
        });
        if self.texts.len() > MAX_TEXTS {
            let extra = self.texts.len() - MAX_TEXTS;
            self.texts.drain(0..extra);
        }
    }

    pub fn update(&mut self, dt_ms: f64) {
        let step = dt_ms.min(MAX_STEP_MS);
        for p in &mut self.particles {
            p.x += p.vx * step;
            p.y += p.vy * step;
            p.vy += GRAVITY * step;
            p.life -= step;
        }
        self.particles.retain(|p| p.life > 0.0);
        for t in &mut self.texts {
            t.y -= 0.03 * step;
            t.life -= step;
        }
        self.texts.retain(|t| t.life > 0.0);
        self.shake_remaining = (self.shake_remaining - step).max(0.0);
        if self.shake_remaining == 0.0 {
            self.shake_intensity = 0.0;
        }
        self.flash_remaining = (self.flash_remaining - step).max(0.0);
    }

    pub fn shake_offset(&self, rng: &mut Rng) -> (f64, f64) {
        if self.shake_remaining <= 0.0 {
            return (0.0, 0.0);
        }
        let falloff = self.shake_remaining / SHAKE_DURATION_MS;
        let amount = self.shake_intensity * falloff;
        ((rng.random() * 2.0 - 1.0) * amount, (rng.random() * 2.0 - 1.0) * amount)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spawns_requested_particles() {
        let mut fx = Effects::default();
        fx.burst(&mut Rng::new(1), 100.0, 100.0, 0xffffff, 12);
        assert_eq!(fx.particle_count(), 12);
    }

    #[test]
    fn retires_particles() {
        let mut fx = Effects::default();
        fx.burst(&mut Rng::new(1), 100.0, 100.0, 0xffffff, 8);
        for _ in 0..40 {
            fx.update(50.0);
        }
        assert_eq!(fx.particle_count(), 0);
    }

    #[test]
    fn keeps_particles_partway() {
        let mut fx = Effects::default();
        fx.burst(&mut Rng::new(1), 100.0, 100.0, 0xffffff, 8);
        fx.update(100.0);
        assert_eq!(fx.particle_count(), 8);
    }

    #[test]
    fn caps_particle_pool() {
        let mut fx = Effects::default();
        let mut rng = Rng::new(1);
        for _ in 0..40 {
            fx.burst(&mut rng, 10.0, 10.0, 0xffffff, 20);
        }
        assert!(fx.particle_count() <= 240);
    }

    #[test]
    fn clamps_huge_step() {
        let mut fx = Effects::default();
        fx.burst(&mut Rng::new(1), 10.0, 10.0, 0xffffff, 8);
        fx.update(100000.0);
        assert_eq!(fx.particle_count(), 8);
    }

    #[test]
    fn no_shake_at_rest() {
        let fx = Effects::default();
        assert_eq!(fx.shake_offset(&mut Rng::new(1)), (0.0, 0.0));
    }

    #[test]
    fn shake_within_intensity() {
        let mut fx = Effects::default();
        fx.shake(10.0);
        let (x, y) = fx.shake_offset(&mut Rng::new(7));
        assert!(x.abs() <= 10.0 && y.abs() <= 10.0);
    }

    #[test]
    fn shake_decays() {
        let mut fx = Effects::default();
        fx.shake(10.0);
        for _ in 0..40 {
            fx.update(50.0);
        }
        assert_eq!(fx.shake_offset(&mut Rng::new(1)), (0.0, 0.0));
    }

    #[test]
    fn stronger_shake_wins() {
        let mut fx = Effects::default();
        fx.shake(4.0);
        fx.shake(12.0);
        fx.update(0.0);
        let (x, _) = fx.shake_offset(&mut Rng::new(3));
        assert!(x.abs() <= 12.0);
    }

    #[test]
    fn flash_fades() {
        let mut fx = Effects::default();
        fx.flash(0xffffff);
        assert!(fx.flash_alpha() > 0.0);
        for _ in 0..40 {
            fx.update(50.0);
        }
        assert_eq!(fx.flash_alpha(), 0.0);
    }

    #[test]
    fn floating_text_retires() {
        let mut fx = Effects::default();
        fx.float_text(50.0, 50.0, "MINE!", 0xff00ff);
        assert_eq!(fx.text_count(), 1);
        for _ in 0..40 {
            fx.update(50.0);
        }
        assert_eq!(fx.text_count(), 0);
    }

    #[test]
    fn floating_text_capped() {
        let mut fx = Effects::default();
        for _ in 0..40 {
            fx.float_text(10.0, 10.0, "x2", 0xffffff);
        }
        assert!(fx.text_count() <= 12);
    }
}
