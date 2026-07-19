import { describe, it, expect } from "vitest";
import { SPRITES, Sprite } from "./sprites";

const entries = Object.entries(SPRITES) as [string, Sprite][];

describe("sprite data", () => {
  it("exposes every sprite the entities need", () => {
    expect(Object.keys(SPRITES).sort()).toEqual(
      ["ghost", "heart", "plant", "star", "starBright", "tetris", "wall", "win"].sort()
    );
  });

  it.each(entries)("%s has rectangular rows", (_name, sprite) => {
    const width = sprite.rows[0].length;
    expect(sprite.rows.length).toBeGreaterThan(0);
    for (const row of sprite.rows) {
      expect(row.length).toBe(width);
    }
  });

  it.each(entries)("%s only uses characters it has colors for", (_name, sprite) => {
    for (const row of sprite.rows) {
      for (const char of row) {
        if (char === ".") continue;
        expect(sprite.palette[char]).toBeDefined();
      }
    }
  });

  it.each(entries)("%s palette holds hex colors", (_name, sprite) => {
    for (const color of Object.values(sprite.palette)) {
      expect(color).toMatch(/^#[0-9a-fA-F]{6}$/);
    }
  });

  it("draws the star twinkle from one shared grid", () => {
    expect(SPRITES.starBright.rows).toBe(SPRITES.star.rows);
    expect(SPRITES.starBright.palette).not.toEqual(SPRITES.star.palette);
  });
});
