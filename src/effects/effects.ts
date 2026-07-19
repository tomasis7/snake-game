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
      : (this.flashRemaining / FLASH_DURATION_MS) * 140;
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
  }

  update(dtMs: number): void {
    for (const particle of this.particles) {
      particle.x += particle.vx * dtMs;
      particle.y += particle.vy * dtMs;
      particle.vy += GRAVITY * dtMs;
      particle.life -= dtMs;
    }
    this.particles = this.particles.filter((p) => p.life > 0);

    for (const text of this.texts) {
      text.y -= 0.03 * dtMs;
      text.life -= dtMs;
    }
    this.texts = this.texts.filter((t) => t.life > 0);

    this.shakeRemaining = Math.max(0, this.shakeRemaining - dtMs);
    if (this.shakeRemaining === 0) {
      this.shakeIntensity = 0;
    }

    this.flashRemaining = Math.max(0, this.flashRemaining - dtMs);
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
