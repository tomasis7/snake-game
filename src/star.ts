import { Entity } from "./entity";
import { SPRITES } from "./art/sprites";
import { drawSprite } from "./art/drawsprite";

export class Star extends Entity {
  constructor(x: number, y: number) {
    super(
      createVector(x, y),
      createVector(32, 32),
      0,
      0,
      createVector(0, 0),
      images.star
    );
  }

  draw(): void {
    if (this.isRemoved) return;
    // Twinkle: swap to the hotter palette twice a second.
    const frame = millis() % 800 < 400 ? SPRITES.star : SPRITES.starBright;
    drawSprite(frame, this.position.x, this.position.y, this.size.x, this.size.y);
  }
}
