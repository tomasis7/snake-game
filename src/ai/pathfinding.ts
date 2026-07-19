// Pure grid AI for the robot player. No p5/browser globals — unit-tested in node.

export interface GridPos {
  col: number;
  row: number;
}

export interface Dir {
  dx: number;
  dy: number;
}

export interface Pickup {
  pos: GridPos;
  value: number;
}

export interface AIWorld {
  blocked: Set<string>;
  minCol: number;
  maxCol: number;
  minRow: number;
  maxRow: number;
}

// Order matters: fallbackDir picks the first free one, so rightward comes first.
const DIRS: Dir[] = [
  { dx: 1, dy: 0 },
  { dx: 0, dy: -1 },
  { dx: 0, dy: 1 },
  { dx: -1, dy: 0 },
];

export function cellKey(col: number, row: number): string {
  return `${col},${row}`;
}

function inBounds(world: AIWorld, pos: GridPos): boolean {
  return (
    pos.col >= world.minCol &&
    pos.col <= world.maxCol &&
    pos.row >= world.minRow &&
    pos.row <= world.maxRow
  );
}

function isFree(world: AIWorld, pos: GridPos): boolean {
  return inBounds(world, pos) && !world.blocked.has(cellKey(pos.col, pos.row));
}

function isReverse(a: Dir, b: Dir): boolean {
  return a.dx === -b.dx && a.dy === -b.dy && (a.dx !== 0 || a.dy !== 0);
}

// Shortest path via BFS; returns the FIRST step of that path, or null if
// unreachable. The first step never reverses currentDir (snakes can't 180).
export function bfsFirstStep(
  world: AIWorld,
  start: GridPos,
  target: GridPos,
  currentDir: Dir
): Dir | null {
  const targetKey = cellKey(target.col, target.row);
  if (cellKey(start.col, start.row) === targetKey) return null;
  if (!isFree(world, target)) return null;

  interface Node {
    pos: GridPos;
    first: Dir;
  }
  const visited = new Set<string>([cellKey(start.col, start.row)]);
  const queue: Node[] = [];

  for (const d of DIRS) {
    if (isReverse(d, currentDir)) continue;
    const next = { col: start.col + d.dx, row: start.row + d.dy };
    const k = cellKey(next.col, next.row);
    if (k === targetKey) return d;
    if (!isFree(world, next)) continue;
    visited.add(k);
    queue.push({ pos: next, first: d });
  }

  let head = 0;
  while (head < queue.length) {
    const { pos, first } = queue[head++];
    for (const d of DIRS) {
      const next = { col: pos.col + d.dx, row: pos.row + d.dy };
      const k = cellKey(next.col, next.row);
      if (visited.has(k)) continue;
      if (k === targetKey) return first;
      if (!isFree(world, next)) continue;
      visited.add(k);
      queue.push({ pos: next, first });
    }
  }
  return null;
}

// Any non-lethal direction, preferring rightward (DIRS order). If everything
// is blocked, keep going — dying forward beats an invalid turn.
export function fallbackDir(
  world: AIWorld,
  start: GridPos,
  currentDir: Dir
): Dir {
  for (const d of DIRS) {
    if (isReverse(d, currentDir)) continue;
    if (isFree(world, { col: start.col + d.dx, row: start.row + d.dy })) {
      return d;
    }
  }
  return currentDir;
}

// Target choice: pickups scored by manhattanDistance / value (lower = better).
// Pickups near the rival's head count double — contest what the human wants.
// Pickups more than 2 columns behind are ignored (the camera scrolls right).
export function decideDirection(
  world: AIWorld,
  start: GridPos,
  currentDir: Dir,
  pickups: Pickup[],
  rivalHead?: GridPos
): Dir {
  const scored = pickups
    .filter((p) => p.pos.col >= start.col - 2)
    .map((p) => {
      let value = p.value;
      if (rivalHead) {
        const rivalDist =
          Math.abs(p.pos.col - rivalHead.col) +
          Math.abs(p.pos.row - rivalHead.row);
        if (rivalDist <= 8) value *= 2;
      }
      const dist =
        Math.abs(p.pos.col - start.col) + Math.abs(p.pos.row - start.row);
      return { pos: p.pos, score: dist / value };
    })
    .sort((a, b) => a.score - b.score);

  for (const t of scored.slice(0, 4)) {
    const step = bfsFirstStep(world, start, t.pos, currentDir);
    if (step) return step;
  }

  const rightTarget = { col: world.maxCol, row: start.row };
  const step = bfsFirstStep(world, start, rightTarget, currentDir);
  return step ?? fallbackDir(world, start, currentDir);
}
