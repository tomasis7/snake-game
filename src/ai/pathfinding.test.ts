import { describe, it, expect } from "vitest";
import {
  AIWorld,
  cellKey,
  bfsFirstStep,
  fallbackDir,
  decideDirection,
} from "./pathfinding";

function makeWorld(blockedCells: [number, number][] = []): AIWorld {
  const blocked = new Set<string>();
  for (const [col, row] of blockedCells) blocked.add(cellKey(col, row));
  return { blocked, minCol: 0, maxCol: 20, minRow: 0, maxRow: 10 };
}

const RIGHT = { dx: 1, dy: 0 };

describe("bfsFirstStep", () => {
  it("steps right toward a target on the same row", () => {
    const world = makeWorld();
    const step = bfsFirstStep(world, { col: 2, row: 5 }, { col: 8, row: 5 }, RIGHT);
    expect(step).toEqual({ dx: 1, dy: 0 });
  });

  it("routes around a wall between start and target", () => {
    // Vertical wall at col 4, rows 3..7; start row 5 -> must go up or down first
    const wall: [number, number][] = [];
    for (let row = 3; row <= 7; row++) wall.push([4, row]);
    const world = makeWorld(wall);
    const step = bfsFirstStep(world, { col: 3, row: 5 }, { col: 8, row: 5 }, RIGHT);
    expect(step).not.toBeNull();
    expect(step!.dx === 0 && (step!.dy === 1 || step!.dy === -1)).toBe(true);
  });

  it("returns null when the target is unreachable", () => {
    // Box the start completely
    const world = makeWorld([
      [4, 5], [6, 5], [5, 4], [5, 6],
    ]);
    const step = bfsFirstStep(world, { col: 5, row: 5 }, { col: 10, row: 5 }, RIGHT);
    expect(step).toBeNull();
  });

  it("never reverses direction", () => {
    // Target directly behind while moving right
    const world = makeWorld();
    const step = bfsFirstStep(world, { col: 5, row: 5 }, { col: 2, row: 5 }, RIGHT);
    // Must not be { dx: -1, dy: 0 } as the FIRST step
    expect(step).not.toEqual({ dx: -1, dy: 0 });
  });

  it("returns null when the target cell itself is blocked", () => {
    const world = makeWorld([[8, 5]]);
    const step = bfsFirstStep(world, { col: 5, row: 5 }, { col: 8, row: 5 }, RIGHT);
    expect(step).toBeNull();
  });
});

describe("fallbackDir", () => {
  it("prefers moving right when free", () => {
    const world = makeWorld();
    expect(fallbackDir(world, { col: 5, row: 5 }, RIGHT)).toEqual({ dx: 1, dy: 0 });
  });

  it("dodges when right is blocked", () => {
    const world = makeWorld([[6, 5]]);
    const d = fallbackDir(world, { col: 5, row: 5 }, RIGHT);
    expect(d.dx).toBe(0);
    expect(Math.abs(d.dy)).toBe(1);
  });

  it("keeps current direction when completely boxed in", () => {
    const world = makeWorld([[4, 5], [6, 5], [5, 4], [5, 6]]);
    expect(fallbackDir(world, { col: 5, row: 5 }, RIGHT)).toEqual(RIGHT);
  });
});

describe("decideDirection", () => {
  it("heads toward the only pickup", () => {
    const world = makeWorld();
    const d = decideDirection(world, { col: 2, row: 5 }, RIGHT, [
      { pos: { col: 2, row: 2 }, value: 3 },
    ]);
    expect(d).toEqual({ dx: 0, dy: -1 });
  });

  it("prefers a star (value 3) over a slightly closer heart (value 1)", () => {
    const world = makeWorld();
    // Heart at distance 4 (score 4/1=4), star at distance 6 (score 6/3=2) -> star wins
    const d = decideDirection(world, { col: 5, row: 5 }, RIGHT, [
      { pos: { col: 5, row: 9 }, value: 1 },
      { pos: { col: 11, row: 5 }, value: 3 },
    ]);
    expect(d).toEqual({ dx: 1, dy: 0 });
  });

  it("doubles the value of pickups near the rival head", () => {
    const world = makeWorld();
    // Two equal hearts; the one near the rival becomes contested (value x2) and wins
    const d = decideDirection(
      world,
      { col: 10, row: 5 },
      RIGHT,
      [
        { pos: { col: 10, row: 1 }, value: 1 },
        { pos: { col: 10, row: 9 }, value: 1 },
      ],
      { col: 10, row: 10 } // rival next to the bottom heart
    );
    expect(d).toEqual({ dx: 0, dy: 1 });
  });

  it("ignores pickups far behind and pushes right instead", () => {
    const world = makeWorld();
    const d = decideDirection(world, { col: 10, row: 5 }, RIGHT, [
      { pos: { col: 2, row: 5 }, value: 3 }, // 8 cols behind — must be ignored
    ]);
    expect(d).toEqual({ dx: 1, dy: 0 });
  });
});
