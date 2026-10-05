// Exports the TypeScript game's levels and pixel-art sprites into plain text
// files that the C++ and Rust ports both load, so the data has one source.
// Run from the repo root: node vulkan/tools/export_data.mjs
import { readFileSync, writeFileSync, mkdirSync } from "node:fs";
import { SPRITES } from "../../src/art/sprites.ts";

const out = "vulkan/assets";
mkdirSync(`${out}/levels`, { recursive: true });

// Levels: `this.levelN = [[...], ...];` in levelfactory.ts. One text line per
// row, one digit per 32px tile (0 empty, 1 block, 2 star, 3 heart, 4 plant,
// 5 ghost, 6 tetris block, 7 win block).
const src = readFileSync("src/levelfactory.ts", "utf8");
for (const n of [1, 2, 3]) {
  const m = src.match(new RegExp(`this\\.level${n} = (\\[[\\s\\S]*?\\]);`));
  if (!m) throw new Error(`level${n} not found`);
  const rows = JSON.parse(m[1].replace(/,\s*\]/g, "]"));
  const width = Math.max(...rows.map((r) => r.length));
  const lines = rows.map((r) => r.join("").padEnd(width, "0"));
  writeFileSync(`${out}/levels/level${n}.txt`, lines.join("\n") + "\n");
  console.log(`level${n}: ${rows.length} rows x ${width} cols`);
}

// Sprites: one block per sprite.
//   sprite <name> <cols> <rows>
//   <char> #rrggbb        (one line per palette entry)
//   rows
//   <row text>            ('.' = transparent)
//   end
let text = "";
for (const [name, s] of Object.entries(SPRITES)) {
  text += `sprite ${name} ${s.rows[0].length} ${s.rows.length}\n`;
  for (const [ch, color] of Object.entries(s.palette)) text += `${ch} ${color}\n`;
  text += "rows\n" + s.rows.join("\n") + "\nend\n";
}
writeFileSync(`${out}/sprites.txt`, text);
console.log(`sprites: ${Object.keys(SPRITES).join(", ")}`);
