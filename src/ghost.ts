import { Entity } from "./entity";
import { SPRITES } from "./art/sprites";
import { drawSprite } from "./art/drawsprite";

export class Ghost extends Entity {
  constructor(x: number, y: number) {
    super(
      createVector(x, y),
      createVector(50, 50),
      0.3,
      0.3,
      createVector(0, 0),
      images.ghost
    );
  }

  draw(): void {
    if (this.isRemoved) return;
    // Bob: a slow 3px vertical float, independent of its drift velocity.
    const bob = Math.sin(millis() / 400) * 3;
    drawSprite(
      SPRITES.ghost,
      this.position.x,
      this.position.y + bob,
      this.size.x,
      this.size.y
    );
  }

  update(): void {
    this.position.add(this.velocity);

    if (this.position.x < 0 || this.position.x > width) {
      this.velocity.x *= -1;
    }
    if (this.position.y < 0 || this.position.y > height) {
      this.velocity.y *= -1;
    }
  }
}
