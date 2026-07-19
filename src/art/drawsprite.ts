import { Sprite } from "./sprites";

// Paints a sprite grid as filled rectangles, centered on (cx, cy) and scaled
// to fill w x h. Cells overlap by a hair so neighbouring pixels of the same
// colour don't show seams at fractional scales.
export function drawSprite(
  sprite: Sprite,
  cx: number,
  cy: number,
  w: number,
  h: number
): void {
  const cols = sprite.rows[0].length;
  const rows = sprite.rows.length;
  const pixelWidth = w / cols;
  const pixelHeight = h / rows;
  const left = cx - w / 2;
  const top = cy - h / 2;

  push();
  noStroke();
  rectMode(CORNER);
  for (let row = 0; row < rows; row++) {
    const line = sprite.rows[row];
    for (let col = 0; col < cols; col++) {
      const char = line[col];
      if (char === ".") continue;
      fill(sprite.palette[char]);
      rect(
        left + col * pixelWidth,
        top + row * pixelHeight,
        pixelWidth + 0.5,
        pixelHeight + 0.5
      );
    }
  }
  pop();
}
