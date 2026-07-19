import { Entity } from "./entity";
import { SPRITES } from "./art/sprites";
import { drawSprite } from "./art/drawsprite";

export class Plant extends Entity {
  constructor(x: number, y: number) {
    super(
      createVector(x, y),
      createVector(32, 64),
      0,
      0,
      createVector(0, 0),
      images.Plant
    );
  }

  draw(): void {
    if (this.isRemoved) return;
    drawSprite(SPRITES.plant, this.position.x, this.position.y, this.size.x, this.size.y);
  }
}
