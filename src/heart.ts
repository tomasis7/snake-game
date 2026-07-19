import { Entity } from "./entity";
import { SPRITES } from "./art/sprites";
import { drawSprite } from "./art/drawsprite";

export class Heart extends Entity {
  private pulseScale: number;
  private pulseSpeed: number;

  constructor(x: number, y: number) {
    super(
      createVector(x, y),
      createVector(32, 32),
      0,
      0,
      createVector(0, 0),
      images.heart
    );
    this.pulseScale = 1;
    this.pulseSpeed = 0.01;
  }

  draw(): void {
    if (this.isRemoved) return;
    drawSprite(
      SPRITES.heart,
      this.position.x,
      this.position.y,
      this.size.x * this.pulseScale,
      this.size.y * this.pulseScale
    );
  }

  update(): void {
    super.update();
    const newPulseScale = 1 + 0.1 * Math.sin(millis() * this.pulseSpeed);
    this.pulseScale = newPulseScale;
  }
}
