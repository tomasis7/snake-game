import { Entity } from "./entity";
import { SPRITES } from "./art/sprites";
import { drawSprite } from "./art/drawsprite";

export class WinBlock extends Entity {
  constructor(x: number, y: number) {
    super(
      createVector(x, y),
      createVector(32, 32),
      0,
      0,
      createVector(0, 0),
      images.WinBlock
    );
  }

  draw(): void {
    if (this.isRemoved) return;
    drawSprite(SPRITES.win, this.position.x, this.position.y, this.size.x, this.size.y);
  }
}
