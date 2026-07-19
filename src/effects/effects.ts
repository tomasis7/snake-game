// Visual feedback: particle bursts, screen shake, screen flash, floating text.
// Everything except drawWorld/drawOverlay is p5-free so it can be unit-tested.

interface Particle {
  x: number;
  y: number;
  vx: number;
  vy: number;
  life: number;
  maxLife: number;
  color: string;
  size: number;
}

interface FloatingText {
  x: number;
  y: number;
  text: string;
  color: string;
  life: number;
  maxLife: number;
}

const PARTICLE_LIFE_MS = 600;
const TEXT_LIFE_MS = 1200;
const SHAKE_DURATION_MS = 350;
const FLASH_DURATION_MS = 250;
const GRAVITY = 0.0006;
const MAX_PARTICLES = 240;
const MAX_TEXTS = 12;
// A single long frame (slow device, tab refocus) must not consume an entire
// effect's lifetime at once, or all feedback vanishes in one tick.
const MAX_STEP_MS = 100;

export class Effects {
  private particles: Particle[] = [];
  private texts: FloatingText[] = [];
  private shakeIntensity: number = 0;
  private shakeRemaining: number = 0;
  private flashRemaining: number = 0;
  private flashColor: string = "#ffffff";

  get particleCount(): number {
    return this.particles.length;
  }

  get textCount(): number {
    return this.texts.length;
  }

  get flashAlpha(): number {
    return this.flashRemaining <= 0
      ? 0
      : (this.flashRemaining / FLASH_DURATION_MS) * 70;
  }

  burst(x: number, y: number, color: string, count: number = 14): void {
    for (let i = 0; i < count; i++) {
      const angle = (Math.PI * 2 * i) / count + Math.random() * 0.4;
      const speed = 0.06 + Math.random() * 0.12;
      this.particles.push({
        x,
        y,
        vx: Math.cos(angle) * speed,
        vy: Math.sin(angle) * speed,
        life: PARTICLE_LIFE_MS,
        maxLife: PARTICLE_LIFE_MS,
        color,
        size: 3 + Math.random() * 3,
      });
    }
    if (this.particles.length > MAX_PARTICLES) {
      this.particles.splice(0, this.particles.length - MAX_PARTICLES);
    }
  }

  shake(intensity: number): void {
    // A weaker shake never cuts one already in progress short.
    this.shakeIntensity = Math.max(this.shakeIntensity, intensity);
    this.shakeRemaining = Math.max(this.shakeRemaining, SHAKE_DURATION_MS);
  }

  flash(color: string): void {
    this.flashColor = color;
    this.flashRemaining = FLASH_DURATION_MS;
  }

  floatText(x: number, y: number, text: string, color: string): void {
    this.texts.push({ x, y, text, color, life: TEXT_LIFE_MS, maxLife: TEXT_LIFE_MS });
    if (this.texts.length > MAX_TEXTS) {
      this.texts.splice(0, this.texts.length - MAX_TEXTS);
    }
  }

  update(dtMs: number): void {
    const step = Math.min(dtMs, MAX_STEP_MS);
    for (const particle of this.particles) {
      particle.x += particle.vx * step;
      particle.y += particle.vy * step;
      particle.vy += GRAVITY * step;
      particle.life -= step;
    }
    this.particles = this.particles.filter((p) => p.life > 0);

    for (const text of this.texts) {
      text.y -= 0.03 * step;
      text.life -= step;
    }
    this.texts = this.texts.filter((t) => t.life > 0);

    this.shakeRemaining = Math.max(0, this.shakeRemaining - step);
    if (this.shakeRemaining === 0) {
      this.shakeIntensity = 0;
    }

    this.flashRemaining = Math.max(0, this.flashRemaining - step);
  }

  shakeOffset(): { x: number; y: number } {
    if (this.shakeRemaining <= 0) {
      return { x: 0, y: 0 };
    }
    const falloff = this.shakeRemaining / SHAKE_DURATION_MS;
    const amount = this.shakeIntensity * falloff;
    return {
      x: (Math.random() * 2 - 1) * amount,
      y: (Math.random() * 2 - 1) * amount,
    };
  }

  // World space: called inside the camera translate.
  drawWorld(): void {
    push();
    noStroke();
    rectMode(CENTER);
    for (const particle of this.particles) {
      const fade = particle.life / particle.maxLife;
      const shade = color(particle.color);
      shade.setAlpha(255 * fade);
      fill(shade);
      rect(particle.x, particle.y, particle.size, particle.size);
    }

    textFont(customFont);
    textAlign(CENTER, CENTER);
    for (const item of this.texts) {
      const fade = item.life / item.maxLife;
      const shade = color(item.color);
      shade.setAlpha(255 * fade);
      fill(shade);
      textSize(16);
      text(item.text, item.x, item.y);
    }
    pop();
  }

  // Screen space: called after the camera translate is popped.
  drawOverlay(): void {
    const alpha = this.flashAlpha;
    if (alpha <= 0) return;
    push();
    noStroke();
    const shade = color(this.flashColor);
    shade.setAlpha(alpha);
    fill(shade);
    rectMode(CORNER);
    rect(0, 0, width, height);
    pop();
  }
}
