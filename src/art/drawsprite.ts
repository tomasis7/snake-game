import { Sprite } from "./sprites";

// Sprites are rasterized once into an offscreen buffer and then blitted, so a
// 16x16 grid costs one image() call per frame instead of 256 rect() calls.
// Buffers are cached per sprite object; SCALE gives enough resolution that
// blitting to the game's 32px cells stays crisp.
const SCALE = 4;

const cache = new Map<Sprite, p5.Graphics>();

function rasterize(sprite: Sprite): p5.Graphics {
  const cols = sprite.rows[0].length;
  const rows = sprite.rows.length;
  const buffer = createGraphics(cols * SCALE, rows * SCALE);
  buffer.noStroke();
  buffer.clear();
  for (let row = 0; row < rows; row++) {
    const line = sprite.rows[row];
    for (let col = 0; col < cols; col++) {
      const char = line[col];
      if (char === ".") continue;
      buffer.fill(sprite.palette[char]);
      buffer.rect(col * SCALE, row * SCALE, SCALE, SCALE);
    }
  }
  return buffer;
}

// Paints a sprite centered on (cx, cy) and scaled to fill w x h.
export function drawSprite(
  sprite: Sprite,
  cx: number,
  cy: number,
  w: number,
  h: number
): void {
  let buffer = cache.get(sprite);
  if (!buffer) {
    buffer = rasterize(sprite);
    cache.set(sprite, buffer);
  }
  push();
  imageMode(CENTER);
  image(buffer, cx, cy, w, h);
  pop();
}
