import { Entity } from "./entity";
import { SPRITES } from "./art/sprites";
import { drawSprite } from "./art/drawsprite";

export class TetrisBlock extends Entity {
  constructor(x: number, y: number) {
    super(
      createVector(x, y),
      createVector(32, 32),
      0,
      0,
      createVector(0, 0),
      images.tetrisBlock
    );
  }

  draw(): void {
    if (this.isRemoved) return;
    drawSprite(SPRITES.tetris, this.position.x, this.position.y, this.size.x, this.size.y);
  }
}
