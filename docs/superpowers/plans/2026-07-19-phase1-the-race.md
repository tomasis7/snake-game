# Phase 1 "The Race" Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a BFS-driven robot rival snake, a 1-Player/2-Player mode select, a 3-level progression with per-level difficulty, and a results screen with a highest-score-wins rule.

**Architecture:** Pure, p5-free AI logic lives in `src/ai/pathfinding.ts` (unit-tested with vitest). `RobotPlayer extends Player` and overrides only the input seam (`handleInput`). `Game` owns run flow (`startRun`/`startLevel`) and a `Progress` tracker; `GameBoard` reports level-end via callback instead of `CollisionManager` changing screens itself.

**Tech Stack:** TypeScript, p5.js (global mode — p5 functions like `createVector`, `width`, `deltaTime` are bare globals, NOT imports), Vite, vitest (added in Task 1).

## Global Constraints

- Spec: `docs/superpowers/specs/2026-07-19-robot-rival-levels-design.md`.
- p5 runs in **global mode**: never import p5 functions; they exist as globals (`createVector`, `width`, `height`, `deltaTime`, `text`, `push`, `pop`, etc.). Globals `game`, `sounds`, `music`, `images`, `customFont` are declared in `global.d.ts`.
- Grid cell size is 32 px everywhere. World coordinates are pixels; grid coordinates are `col = Math.floor(x / 32)`, `row = Math.floor(y / 32)`.
- `src/ai/pathfinding.ts` and `src/progress.ts` MUST NOT reference any p5 or browser global (except the guarded `localStorage` default in progress.ts) — they are unit-tested in node.
- Snakes cannot reverse 180°; the AI must respect this.
- Win rule: **highest score when the level ends** — regardless of who triggered the end.
- Level count is 3. Per-level settings: Level 1 `{scrollSpeed: 1.5, robotMistakeChance: 0.25}`, Level 2 `{scrollSpeed: 2.0, robotMistakeChance: 0.10}`, Level 3 `{scrollSpeed: 2.5, robotMistakeChance: 0}`.
- Robot is Player 2 (magenta `#FF00FF`) in 1-Player mode; 2-Player mode is exactly today's arrows-vs-WASD game.
- Existing code compiles after EVERY task: run `npm run build` before each commit.
- Commit messages end with: `Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>`

---

### Task 1: Pure AI pathfinding module (+ vitest setup)

**Files:**
- Create: `src/ai/pathfinding.ts`
- Test: `src/ai/pathfinding.test.ts`
- Modify: `package.json` (add vitest + test script)

**Interfaces:**
- Consumes: nothing (pure module, zero imports).
- Produces (used by Task 2's RobotPlayer):
  - `interface GridPos { col: number; row: number }`
  - `interface Dir { dx: number; dy: number }` (exactly one of dx/dy is ±1)
  - `interface Pickup { pos: GridPos; value: number }`
  - `interface AIWorld { blocked: Set<string>; minCol: number; maxCol: number; minRow: number; maxRow: number }`
  - `cellKey(col: number, row: number): string`
  - `decideDirection(world: AIWorld, start: GridPos, currentDir: Dir, pickups: Pickup[], rivalHead?: GridPos): Dir`
  - `bfsFirstStep(world: AIWorld, start: GridPos, target: GridPos, currentDir: Dir): Dir | null`
  - `fallbackDir(world: AIWorld, start: GridPos, currentDir: Dir): Dir`

- [ ] **Step 1: Install vitest and add test script**

```bash
npm install --save-dev vitest
```

In `package.json` `"scripts"`, add:

```json
"test": "vitest run"
```

- [ ] **Step 2: Write the failing tests**

Create `src/ai/pathfinding.test.ts`:

```typescript
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
```

- [ ] **Step 3: Run tests to verify they fail**

Run: `npx vitest run src/ai/pathfinding.test.ts`
Expected: FAIL — cannot resolve `./pathfinding`.

- [ ] **Step 4: Implement the module**

Create `src/ai/pathfinding.ts`:

```typescript
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
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `npx vitest run src/ai/pathfinding.test.ts`
Expected: PASS (11 tests).

- [ ] **Step 6: Verify the app still builds**

Run: `npm run build`
Expected: success, no TypeScript errors.

- [ ] **Step 7: Commit**

```bash
git add src/ai/pathfinding.ts src/ai/pathfinding.test.ts package.json package-lock.json
git commit -m "feat: add pure BFS pathfinding module for robot AI (with vitest)

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

### Task 2: Player override seam + RobotPlayer

**Files:**
- Modify: `src/player.ts` (make `handleInput`/`nextDirection` protected, add `displayName`)
- Create: `src/robotplayer.ts`

**Interfaces:**
- Consumes (from Task 1, `src/ai/pathfinding.ts`): `decideDirection(world, start, currentDir, pickups, rivalHead?)`, `cellKey(col, row)`, types `AIWorld`, `GridPos`, `Dir`, `Pickup`.
- Consumes (existing): `Player` from `src/player.ts` — constructor `(position: p5.Vector, playerNumber: number, trailFillColor: string, trailStrokeColor: string, keyBindings: KeyBindings)`; fields `trail: p5.Vector[]`, `direction: p5.Vector` (pixels, e.g. `(32, 0)`), `nextDirection`.
- Produces (used by Task 5's GameBoard):
  - `class RobotPlayer extends Player` with constructor `(position: p5.Vector, playerNumber: number, trailFillColor: string, trailStrokeColor: string, mistakeChance: number)`
  - `robot.setContext(ctx: RobotContext): void` where `interface RobotContext { entities: Entity[]; cameraOffset: number; otherTrails: p5.Vector[][] }`
  - `Player.displayName: string` — `"PLAYER 1"` / `"PLAYER 2"` by default, `"ROBOT"` on RobotPlayer (used by ScoreManager/results later).

- [ ] **Step 1: Open the override seam in Player**

In `src/player.ts`, change (line 16):

```typescript
  private nextDirection: p5.Vector;
```

to:

```typescript
  protected nextDirection: p5.Vector;
```

Change (line 72):

```typescript
  private handleInput(): void {
```

to:

```typescript
  protected handleInput(): void {
```

Add a public field after `public playerNumber: number;` (line 12):

```typescript
  public displayName: string;
```

and in the constructor, after `this.playerNumber = playerNumber;` add:

```typescript
    this.displayName = `PLAYER ${playerNumber}`;
```

- [ ] **Step 2: Create RobotPlayer**

Create `src/robotplayer.ts`:

```typescript
import { Player } from "./player";
import { Entity } from "./entity";
import { Star } from "./star";
import { Heart } from "./heart";
import { WinBlock } from "./winBlock";
import {
  AIWorld,
  Pickup,
  GridPos,
  cellKey,
  decideDirection,
} from "./ai/pathfinding";

export interface RobotContext {
  entities: Entity[];
  cameraOffset: number;
  otherTrails: p5.Vector[][];
}

const CELL = 32;

function toGrid(x: number, y: number): GridPos {
  return { col: Math.floor(x / CELL), row: Math.floor(y / CELL) };
}

export class RobotPlayer extends Player {
  private mistakeChance: number;
  private context: RobotContext | null = null;
  private thinkTimer: number = 0;

  constructor(
    position: p5.Vector,
    playerNumber: number,
    trailFillColor: string,
    trailStrokeColor: string,
    mistakeChance: number
  ) {
    // Key codes -1: no real keys ever match, the robot ignores the keyboard.
    super(position, playerNumber, trailFillColor, trailStrokeColor, {
      UP: -1,
      DOWN: -1,
      LEFT: -1,
      RIGHT: -1,
    });
    this.mistakeChance = mistakeChance;
    this.displayName = "ROBOT";
  }

  public setContext(context: RobotContext): void {
    this.context = context;
  }

  protected handleInput(): void {
    if (!this.context) return;

    this.thinkTimer += deltaTime;
    if (this.thinkTimer < 200) return;
    this.thinkTimer = 0;

    // Difficulty knob: on a "mistake" tick the robot skips thinking and drifts.
    if (Math.random() < this.mistakeChance) return;

    const { world, pickups } = this.buildWorld();
    const head = toGrid(this.trail[0].x, this.trail[0].y);
    const currentDir = {
      dx: Math.sign(this.direction.x),
      dy: Math.sign(this.direction.y),
    };
    // Contested targeting is the perfect-robot (level 3) behavior only.
    const rivalTrail = this.context.otherTrails[0];
    const rivalHead =
      this.mistakeChance === 0 && rivalTrail?.length
        ? toGrid(rivalTrail[0].x, rivalTrail[0].y)
        : undefined;

    const next = decideDirection(world, head, currentDir, pickups, rivalHead);
    this.nextDirection = createVector(next.dx * CELL, next.dy * CELL);
  }

  private buildWorld(): { world: AIWorld; pickups: Pickup[] } {
    const { entities, cameraOffset, otherTrails } = this.context!;
    const minCol = Math.max(0, Math.floor(cameraOffset / CELL));
    const maxCol = minCol + Math.ceil(width / CELL) + 4;
    const world: AIWorld = {
      blocked: new Set<string>(),
      minCol,
      maxCol,
      minRow: 0,
      maxRow: Math.floor(height / CELL) - 1,
    };
    const pickups: Pickup[] = [];

    for (const e of entities) {
      if (e.isRemoved) continue;
      const { col, row } = toGrid(e.position.x, e.position.y);
      if (col < minCol - 2 || col > maxCol) continue;
      if (e instanceof Star) {
        pickups.push({ pos: { col, row }, value: 3 });
      } else if (e instanceof Heart) {
        pickups.push({ pos: { col, row }, value: 1 });
      } else if (!(e instanceof WinBlock)) {
        // Blocks, tetris blocks, plants, ghosts: all cells to avoid.
        world.blocked.add(cellKey(col, row));
      }
    }

    for (const trail of otherTrails) {
      for (const segment of trail) {
        const { col, row } = toGrid(segment.x, segment.y);
        world.blocked.add(cellKey(col, row));
      }
    }
    // Own trail (skip the head — that's where we are).
    for (let i = 1; i < this.trail.length; i++) {
      const { col, row } = toGrid(this.trail[i].x, this.trail[i].y);
      world.blocked.add(cellKey(col, row));
    }

    return { world, pickups };
  }
}
```

- [ ] **Step 3: Verify the build**

Run: `npm run build`
Expected: success. Also run `npx vitest run` — Task 1 tests still pass.

- [ ] **Step 4: Commit**

```bash
git add src/player.ts src/robotplayer.ts
git commit -m "feat: add RobotPlayer with BFS brain behind Player input seam

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

### Task 3: Progress tracker + per-level configs

**Files:**
- Create: `src/progress.ts`
- Test: `src/progress.test.ts`
- Modify: `src/levelfactory.ts` (add `LevelConfig` + `getLevelConfig`)

**Interfaces:**
- Consumes: nothing p5-related; `LevelFactory.level1/level2/level3: number[][]` already exist.
- Produces (used by Tasks 4–5):
  - `type GameMode = "onePlayer" | "twoPlayer"` (exported from `src/progress.ts`)
  - `class Progress`: `startRun(mode: GameMode): void`, `mode: GameMode`, `currentLevel: number`, `addLevelScores(score1: number, score2: number): void`, `getTotal(playerNumber: number): number`, `isLastLevel(): boolean`, `finishRun(): { best: number; isNewBest: boolean }`; constructor takes optional `storage: Pick<Storage, "getItem" | "setItem">`
  - `LevelFactory.getLevelConfig(levelNumber: number): LevelConfig` with `interface LevelConfig { layout: number[][]; scrollSpeed: number; robotMistakeChance: number }` (exported)
  - `LevelFactory.LEVEL_COUNT = 3` (public static readonly)

- [ ] **Step 1: Write the failing tests**

Create `src/progress.test.ts`:

```typescript
import { describe, it, expect } from "vitest";
import { Progress } from "./progress";

function fakeStorage(): Pick<Storage, "getItem" | "setItem"> & {
  data: Map<string, string>;
} {
  const data = new Map<string, string>();
  return {
    data,
    getItem: (k: string) => data.get(k) ?? null,
    setItem: (k: string, v: string) => void data.set(k, v),
  };
}

describe("Progress", () => {
  it("accumulates level scores into run totals", () => {
    const p = new Progress(fakeStorage());
    p.startRun("onePlayer");
    p.addLevelScores(100, 80);
    p.addLevelScores(50, 120);
    expect(p.getTotal(1)).toBe(150);
    expect(p.getTotal(2)).toBe(200);
  });

  it("resets totals and level on startRun", () => {
    const p = new Progress(fakeStorage());
    p.startRun("onePlayer");
    p.addLevelScores(100, 80);
    p.startRun("twoPlayer");
    expect(p.getTotal(1)).toBe(0);
    expect(p.currentLevel).toBe(1);
    expect(p.mode).toBe("twoPlayer");
  });

  it("reports last level at level 3", () => {
    const p = new Progress(fakeStorage());
    p.startRun("onePlayer");
    expect(p.isLastLevel()).toBe(false);
    p.currentLevel = 3;
    expect(p.isLastLevel()).toBe(true);
  });

  it("persists a new best total and detects it", () => {
    const storage = fakeStorage();
    const p = new Progress(storage);
    p.startRun("onePlayer");
    p.addLevelScores(300, 10);
    expect(p.finishRun()).toEqual({ best: 300, isNewBest: true });

    const p2 = new Progress(storage);
    p2.startRun("onePlayer");
    p2.addLevelScores(200, 10);
    expect(p2.finishRun()).toEqual({ best: 300, isNewBest: false });
  });
});
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `npx vitest run src/progress.test.ts`
Expected: FAIL — cannot resolve `./progress`.

- [ ] **Step 3: Implement Progress**

Create `src/progress.ts`:

```typescript
// Run-wide progression state (mode, current level, cumulative totals, best
// score persistence). No p5 globals — unit-tested in node.

export type GameMode = "onePlayer" | "twoPlayer";

const BEST_KEY = "furious-snake-best-total";

const noStorage: Pick<Storage, "getItem" | "setItem"> = {
  getItem: () => null,
  setItem: () => {},
};

export class Progress {
  public mode: GameMode = "onePlayer";
  public currentLevel: number = 1;
  private totals: Map<number, number> = new Map();
  private storage: Pick<Storage, "getItem" | "setItem">;

  constructor(storage?: Pick<Storage, "getItem" | "setItem">) {
    this.storage =
      storage ?? (typeof localStorage !== "undefined" ? localStorage : noStorage);
  }

  startRun(mode: GameMode): void {
    this.mode = mode;
    this.currentLevel = 1;
    this.totals.clear();
  }

  addLevelScores(score1: number, score2: number): void {
    this.totals.set(1, (this.totals.get(1) ?? 0) + score1);
    this.totals.set(2, (this.totals.get(2) ?? 0) + score2);
  }

  getTotal(playerNumber: number): number {
    return this.totals.get(playerNumber) ?? 0;
  }

  isLastLevel(): boolean {
    return this.currentLevel >= 3;
  }

  finishRun(): { best: number; isNewBest: boolean } {
    const total = this.getTotal(1);
    const previous = Number(this.storage.getItem(BEST_KEY) ?? "0");
    const isNewBest = total > previous;
    if (isNewBest) {
      this.storage.setItem(BEST_KEY, String(total));
    }
    return { best: Math.max(total, previous), isNewBest };
  }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `npx vitest run src/progress.test.ts`
Expected: PASS (4 tests).

- [ ] **Step 5: Add LevelConfig to LevelFactory**

In `src/levelfactory.ts`, directly after the imports (line 8), add the export:

```typescript
export interface LevelConfig {
  layout: number[][];
  scrollSpeed: number;
  robotMistakeChance: number;
}
```

Inside the `LevelFactory` class, after `public level3: number[][];` (line 14), add:

```typescript
  public static readonly LEVEL_COUNT = 3;
```

At the end of the class (after `createEntitiesForLevel`), add:

```typescript
  public getLevelConfig(levelNumber: number): LevelConfig {
    const configs: LevelConfig[] = [
      { layout: this.level1, scrollSpeed: 1.5, robotMistakeChance: 0.25 },
      { layout: this.level2, scrollSpeed: 2.0, robotMistakeChance: 0.1 },
      { layout: this.level3, scrollSpeed: 2.5, robotMistakeChance: 0 },
    ];
    const index = Math.min(Math.max(levelNumber, 1), LevelFactory.LEVEL_COUNT);
    return configs[index - 1];
  }
```

- [ ] **Step 6: Verify build and all tests**

Run: `npm run build && npx vitest run`
Expected: build succeeds; all tests pass.

- [ ] **Step 7: Commit**

```bash
git add src/progress.ts src/progress.test.ts src/levelfactory.ts
git commit -m "feat: add Progress run tracker and per-level configs

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

### Task 4: Game flow rework — mode select, startRun/startLevel, CountDown fix

**Files:**
- Modify: `src/game.ts` (add `progress`, `startRun`, `startLevel`; remove dead `newGame`)
- Modify: `src/startmenu.ts` (replace difficulty select with mode select)
- Modify: `src/countdown.ts` (callback-only; remove double GameBoard creation)
- Modify: `src/gameboard.ts` (constructor signature ONLY: `(levelNumber: number, mode: GameMode)`)

**Interfaces:**
- Consumes (Task 3): `GameMode`, `Progress` from `./progress`; `LevelFactory.getLevelConfig`.
- Produces (used by Task 5's ResultsScreen):
  - `game.startRun(mode: GameMode): void` — resets progress, starts level 1
  - `game.startLevel(levelNumber: number): void` — countdown then `GameBoard(levelNumber, mode)`
  - `game.progress: Progress` (public)
  - `new StartMenu()` — zero-arg constructor
  - `new CountDown(onComplete: () => void)` — calls `onComplete` exactly once when the countdown hits 0; it no longer creates a GameBoard itself
  - `new GameBoard(levelNumber: number, mode: GameMode)`

- [ ] **Step 1: Rework Game**

Replace the entire contents of `src/game.ts` with:

```typescript
import { GameScreen } from "./gamescreen";
import { StartMenu } from "./startmenu";
import { GameBoard } from "./gameboard";
import { CountDown } from "./countdown";
import { Progress, GameMode } from "./progress";

export class Game {
  private activeScreen: GameScreen[];
  public progress: Progress;

  constructor() {
    this.progress = new Progress();
    this.activeScreen = [new StartMenu()];
  }

  changeScreen(newScreen: GameScreen): void {
    this.activeScreen = [newScreen];
  }

  startRun(mode: GameMode): void {
    this.progress.startRun(mode);
    this.startLevel(1);
  }

  startLevel(levelNumber: number): void {
    this.progress.currentLevel = levelNumber;
    this.changeScreen(
      new CountDown(() => {
        this.changeScreen(new GameBoard(levelNumber, this.progress.mode));
      })
    );
  }

  public update(): void {
    for (const screen of this.activeScreen) {
      screen.update();
    }
  }

  draw(): void {
    for (const screen of this.activeScreen) {
      screen.draw();
    }
  }

  end(): void {}
}
```

- [ ] **Step 2: Rework StartMenu into a mode select**

Replace the entire contents of `src/startmenu.ts` with:

```typescript
import { GameScreen } from "./gamescreen";
import { Button } from "./button";
import { InteractionScreen } from "./interactionscreen";
import { GameMode } from "./progress";

export class StartMenu extends GameScreen {
  startGameButton: Button;
  onePlayerButton: Button;
  twoPlayerButton: Button;
  howToPlayButton: Button;
  selectedMode: GameMode = "onePlayer";

  constructor() {
    super();
    this.startGameButton = new Button(
      "Start Game",
      createVector(width / 2, height / 2 + 125),
      "#515151",
      createVector(350, 50),
      "#45FF8C"
    );

    this.onePlayerButton = new Button(
      "1 Player vs Robot",
      createVector(width / 2, height / 2 - 100),
      "#515151",
      createVector(420, 50),
      "#00FFFF"
    );

    this.twoPlayerButton = new Button(
      "2 Players",
      createVector(width / 2, height / 2 - 25),
      "#515151",
      createVector(420, 50),
      "#FF00FF"
    );

    this.howToPlayButton = new Button(
      "How to play",
      createVector(width / 2, height - 100),
      "#515151",
      createVector(380, 50),
      "#FFFFFF"
    );
  }

  update(): void {
    if (this.onePlayerButton.isClicked()) {
      this.selectedMode = "onePlayer";
    }

    if (this.twoPlayerButton.isClicked()) {
      this.selectedMode = "twoPlayer";
    }

    if (this.startGameButton.isClicked()) {
      userStartAudio();
      if (!music.backgroundMusic.isPlaying()) {
        music.backgroundMusic.loop();
      }
      game.startRun(this.selectedMode);
    }

    if (this.howToPlayButton.isClicked()) {
      game.changeScreen(new InteractionScreen());
    }
  }

  draw(): void {
    background("black");

    push();
    fill("#45FF8C");
    textAlign(CENTER, CENTER);
    textFont(customFont);
    textSize(42);
    text("Furious Snake", width / 2, height / 4 - 100);

    this.onePlayerButton.backgroundColor =
      this.selectedMode === "onePlayer" ? "white" : "#515151";
    this.twoPlayerButton.backgroundColor =
      this.selectedMode === "twoPlayer" ? "white" : "#515151";

    fill("#45FF8C");
    textSize(32);
    text("SELECT MODE", width / 2, height / 4);

    this.startGameButton.draw();
    this.onePlayerButton.draw();
    this.twoPlayerButton.draw();
    this.howToPlayButton.draw();
    pop();
  }
}
```

Note: `userStartAudio` is a p5.sound global, already used the same way in the old file.

- [ ] **Step 3: Fix CountDown**

Replace the entire contents of `src/countdown.ts` with:

```typescript
import { GameScreen } from "./gamescreen";

export class CountDown extends GameScreen {
  private countdownValue: number;
  private lastUpdateTime: number;
  private readonly countdownDuration: number = 3;
  private isComplete: boolean;
  private callback: () => void;

  constructor(callback: () => void) {
    super();
    this.countdownValue = this.countdownDuration;
    this.lastUpdateTime = Date.now();
    this.isComplete = false;
    this.callback = callback;
  }

  update(): void {
    const currentTime = Date.now();
    const deltaSeconds = (currentTime - this.lastUpdateTime) / 1000;
    this.lastUpdateTime = currentTime;

    if (!this.isComplete && this.countdownValue > 0) {
      this.countdownValue -= deltaSeconds;
      if (this.countdownValue <= 0) {
        this.countdownValue = 0;
        this.isComplete = true;
        this.callback();
      }
    }
  }

  draw(): void {
    push();
    background("black");
    fill("#45FF8C");
    textSize(32);
    textAlign(CENTER, CENTER);
    textFont(customFont);
    text("GET READY", width / 2, height / 4);

    fill("#FFFFFF");
    textSize(84);

    const displayNumber = Math.ceil(this.countdownValue);
    if (displayNumber > 0) {
      text(displayNumber.toString(), width / 2, height / 3);
    }
    pop();
  }

  isCountdownComplete(): boolean {
    return this.isComplete;
  }
}
```

- [ ] **Step 4: Update the GameBoard constructor signature (minimal change)**

In `src/gameboard.ts`, add imports at the top:

```typescript
import { GameMode } from "./progress";
```

Replace the constructor's first lines (currently `constructor(level: number[][]) { super();`) and the level usage:

```typescript
  private levelNumber: number;
  private mode: GameMode;

  constructor(levelNumber: number, mode: GameMode) {
    super();
    this.levelNumber = levelNumber;
    this.mode = mode;
```

and replace:

```typescript
    this.levelFactory = new LevelFactory();
    this.entities = this.levelFactory.createEntitiesForLevel(level);
```

with:

```typescript
    this.levelFactory = new LevelFactory();
    const config = this.levelFactory.getLevelConfig(levelNumber);
    this.scrollSpeed = config.scrollSpeed;
    this.entities = this.levelFactory.createEntitiesForLevel(config.layout);
```

(`scrollSpeed` already exists as a field with default 1.5 — the config now sets it. Robot wiring comes in Task 5; both players stay human in this task.)

- [ ] **Step 5: Verify build and tests**

Run: `npm run build && npx vitest run`
Expected: build succeeds (no remaining references to old constructors — `grep -rn "new GameBoard(\|new StartMenu(\|new CountDown(" src/` should show only the new signatures); tests pass.

- [ ] **Step 6: Smoke-test in the browser**

Run: `npm run dev` (background), open the shown URL. Expected: menu shows "1 Player vs Robot"/"2 Players"/"Start Game"; Start runs the countdown once, then level 1 plays with two keyboard-controlled snakes. Stop the dev server after.

- [ ] **Step 7: Commit**

```bash
git add src/game.ts src/startmenu.ts src/countdown.ts src/gameboard.ts
git commit -m "feat: mode select menu and level-based game flow

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

### Task 5: Robot wiring, level-end flow, results screen

**Files:**
- Modify: `src/gameboard.ts` (robot player, context feed, endLevel, HUD)
- Modify: `src/collisionmanager.ts` (onLevelEnd callback replaces showGameOver)
- Modify: `src/scoreManager.ts` (use `player.displayName`)
- Create: `src/resultsscreen.ts`
- Delete: `src/gameOverScreen.ts`

**Interfaces:**
- Consumes: `RobotPlayer`, `RobotContext` (Task 2); `game.progress`, `game.startLevel`, `GameMode`, `Progress.addLevelScores/getTotal/isLastLevel/finishRun` (Tasks 3–4); `StartMenu` zero-arg (Task 4); `ScoreManager.getScore(playerNumber)` (existing); `Player.displayName` (Task 2).
- Produces: `class ResultsScreen extends GameScreen` with constructor `(levelNumber: number, mode: GameMode, score1: number, score2: number)`. `CollisionManager` constructor gains a 5th parameter `onLevelEnd: () => void`.

- [ ] **Step 1: CollisionManager — replace screen changes with a callback**

In `src/collisionmanager.ts`:

1. Remove the import of `GameOverScreen` (line 11).
2. Add the callback field and constructor parameter:

```typescript
  private onLevelEnd: () => void;

  constructor(
    players: Player[],
    entities: Entity[],
    scoreManager: ScoreManager,
    removeEntityCallback: (entity: Entity) => void,
    onLevelEnd: () => void
  ) {
    this.players = players;
    this.entities = entities;
    this.scoreManager = scoreManager;
    this.removeEntityCallback = removeEntityCallback;
    this.onLevelEnd = onLevelEnd;
  }
```

3. Replace every `this.showGameOver(...)` call (in `handleTetrisCollision`, `handleBlockCollision`, `handleWinBlockCollision`, `handlePlantCollision`, `handleGhostCollision`) with:

```typescript
    this.onLevelEnd();
```

4. In `handleWinBlockCollision`, also delete the now-unused line `const otherPlayerNumber = player.playerNumber === 1 ? 2 : 1;`.
5. Delete the whole `private showGameOver(...)` method (lines 242–252).

- [ ] **Step 2: ScoreManager — display names**

In `src/scoreManager.ts` `draw()`, replace:

```typescript
        const textContent = `Player: ${playerNumber} Score: ${score} | Lives: ${player.lives}`;
```

with:

```typescript
        const textContent = `${player.displayName} Score: ${score} | Lives: ${player.lives}`;
```

- [ ] **Step 3: Create ResultsScreen**

Create `src/resultsscreen.ts`:

```typescript
import { GameScreen } from "./gamescreen";
import { Button } from "./button";
import { StartMenu } from "./startmenu";
import { GameMode } from "./progress";

export class ResultsScreen extends GameScreen {
  private levelNumber: number;
  private mode: GameMode;
  private score1: number;
  private score2: number;
  private winner: number; // 1, 2, or 0 for tie
  private isFinal: boolean;
  private finalBest: { best: number; isNewBest: boolean } | null = null;

  private nextLevelButton: Button | null = null;
  private retryButton: Button;
  private menuButton: Button;

  constructor(
    levelNumber: number,
    mode: GameMode,
    score1: number,
    score2: number
  ) {
    super();
    this.levelNumber = levelNumber;
    this.mode = mode;
    this.score1 = score1;
    this.score2 = score2;
    this.winner = score1 > score2 ? 1 : score2 > score1 ? 2 : 0;
    this.isFinal = levelNumber >= 3;

    if (this.isFinal) {
      this.finalBest = game.progress.finishRun();
    }

    const humanAdvances =
      this.mode === "twoPlayer" ? this.winner !== 0 : this.winner === 1;
    if (!this.isFinal && humanAdvances) {
      this.nextLevelButton = new Button(
        "Next Level",
        createVector(width / 2, height / 2 + 140),
        "#515151",
        createVector(300, 50),
        "#45FF8C"
      );
    }

    this.retryButton = new Button(
      "Retry",
      createVector(width / 2, height / 2 + 210),
      "#515151",
      createVector(300, 50),
      "#FDD03C"
    );

    this.menuButton = new Button(
      "Menu",
      createVector(width / 2, height / 2 + 280),
      "#515151",
      createVector(300, 50),
      "#FFFFFF"
    );
  }

  private winnerText(): string {
    if (this.winner === 0) return "IT'S A TIE!";
    if (this.mode === "onePlayer") {
      return this.winner === 1 ? "YOU WIN!" : "ROBOT WINS!";
    }
    return `PLAYER ${this.winner} WINS!`;
  }

  private nameFor(playerNumber: number): string {
    if (this.mode === "onePlayer") {
      return playerNumber === 1 ? "YOU" : "ROBOT";
    }
    return `PLAYER ${playerNumber}`;
  }

  update(): void {
    if (this.nextLevelButton && this.nextLevelButton.isClicked()) {
      game.startLevel(this.levelNumber + 1);
      return;
    }
    if (this.retryButton.isClicked()) {
      game.startLevel(this.levelNumber);
      return;
    }
    if (this.menuButton.isClicked()) {
      game.changeScreen(new StartMenu());
    }
  }

  draw(): void {
    push();
    background("black");
    textFont(customFont);
    textAlign(CENTER, CENTER);

    fill("#45FF8C");
    textSize(28);
    text(
      this.isFinal ? "FINAL RESULTS" : `LEVEL ${this.levelNumber} COMPLETE`,
      width / 2,
      height / 6
    );

    fill("white");
    textSize(48);
    text(this.winnerText(), width / 2, height / 6 + 90);

    textSize(22);
    fill("#00FFFF");
    text(
      `${this.nameFor(1)}  ${this.score1}`,
      width / 2 - 220,
      height / 2 - 40
    );
    fill("#FF00FF");
    text(
      `${this.nameFor(2)}  ${this.score2}`,
      width / 2 + 220,
      height / 2 - 40
    );

    if (this.isFinal && this.finalBest) {
      fill("white");
      textSize(18);
      text(
        `RUN TOTAL  ${this.nameFor(1)} ${game.progress.getTotal(1)}  -  ${this.nameFor(2)} ${game.progress.getTotal(2)}`,
        width / 2,
        height / 2 + 30
      );
      fill("#FDD03C");
      text(
        this.finalBest.isNewBest
          ? `NEW BEST: ${this.finalBest.best}!`
          : `BEST: ${this.finalBest.best}`,
        width / 2,
        height / 2 + 70
      );
    }

    if (this.nextLevelButton) this.nextLevelButton.draw();
    this.retryButton.draw();
    this.menuButton.draw();
    pop();
  }
}
```

- [ ] **Step 4: Wire the GameBoard**

Replace the entire contents of `src/gameboard.ts` with:

```typescript
import { GameScreen } from "./gamescreen";
import { Entity } from "./entity";
import { Player } from "./player";
import { RobotPlayer } from "./robotplayer";
import { LevelFactory } from "./levelfactory";
import { CollisionManager } from "./collisionmanager";
import { ScoreManager } from "./scoreManager";
import { ResultsScreen } from "./resultsscreen";
import { Ghost } from "./ghost";
import { Heart } from "./heart";
import { GameMode } from "./progress";

export class GameBoard extends GameScreen {
  private entities: Entity[];
  private players: Player[];
  private levelFactory: LevelFactory;
  private collisionManager: CollisionManager;
  private scoreManager: ScoreManager;
  private levelNumber: number;
  private mode: GameMode;
  private levelEnded: boolean = false;

  private cameraOffset: number = 0;
  private scrollSpeed: number = 1.5;

  constructor(levelNumber: number, mode: GameMode) {
    super();
    this.levelNumber = levelNumber;
    this.mode = mode;

    this.levelFactory = new LevelFactory();
    const config = this.levelFactory.getLevelConfig(levelNumber);
    this.scrollSpeed = config.scrollSpeed;

    const playerOne = new Player(createVector(128, 192), 1, "#00FFFF", "green", {
      UP: UP_ARROW,
      DOWN: DOWN_ARROW,
      RIGHT: RIGHT_ARROW,
      LEFT: LEFT_ARROW,
    });
    const playerTwo =
      mode === "onePlayer"
        ? new RobotPlayer(
            createVector(128, 576),
            2,
            "#FF00FF",
            "orange",
            config.robotMistakeChance
          )
        : new Player(createVector(128, 576), 2, "#FF00FF", "orange", {
            UP: 87,
            DOWN: 83,
            RIGHT: 68,
            LEFT: 65,
          });
    this.players = [playerOne, playerTwo];

    this.entities = this.levelFactory.createEntitiesForLevel(config.layout);

    this.scoreManager = new ScoreManager(this.players);
    this.collisionManager = new CollisionManager(
      this.players,
      this.entities,
      this.scoreManager,
      this.removeEntity.bind(this),
      this.endLevel.bind(this)
    );
  }

  addEntity(entity: Entity): void {
    if (!(entity instanceof Heart)) {
      this.entities.push(entity);
    }
  }

  removeEntity(entity: Entity): void {
    this.entities = this.entities.filter((e) => e !== entity);
  }

  private endLevel(): void {
    if (this.levelEnded) return;
    this.levelEnded = true;

    const score1 = this.scoreManager.getScore(1);
    const score2 = this.scoreManager.getScore(2);
    game.progress.addLevelScores(score1, score2);
    game.changeScreen(
      new ResultsScreen(this.levelNumber, this.mode, score1, score2)
    );
  }

  public update(): void {
    if (this.levelEnded) return;

    this.cameraOffset += this.scrollSpeed;

    for (const player of this.players) {
      if (player instanceof RobotPlayer) {
        player.setContext({
          entities: this.entities,
          cameraOffset: this.cameraOffset,
          otherTrails: this.players
            .filter((p) => p !== player)
            .map((p) => p.trail),
        });
      }
      player.update();
    }

    for (const entity of this.entities) {
      entity.update();
    }

    this.flyingGhost();

    this.collisionManager.checkCollision();
    this.scoreManager.tickScore();
  }

  private flyingGhost(): void {
    for (const entity of this.entities) {
      if (entity instanceof Ghost) {
        entity.update();
      }
    }
  }

  draw(): void {
    background(0);
    const numBackgrounds = Math.ceil((width + this.cameraOffset) / 1415) + 1;
    for (let i = 0; i < numBackgrounds; i++) {
      image(images.background, i * 1415 - this.cameraOffset, 0, 1415, 800);
    }
    push();
    translate(-this.cameraOffset, 0);

    for (const entity of this.entities) {
      entity.draw();
    }

    for (const player of this.players) {
      player.draw();
    }

    pop();

    this.scoreManager.draw();

    push();
    textFont(customFont);
    textSize(16);
    textAlign(LEFT, CENTER);
    fill("#45FF8C");
    text(`LEVEL ${this.levelNumber} / ${LevelFactory.LEVEL_COUNT}`, 20, 50);
    pop();
  }
}
```

- [ ] **Step 5: Delete the dead GameOverScreen**

```bash
git rm src/gameOverScreen.ts
grep -rn "GameOverScreen" src/
```

Expected: grep finds nothing.

- [ ] **Step 6: Verify build and tests**

Run: `npm run build && npx vitest run`
Expected: build succeeds; all tests pass.

- [ ] **Step 7: Commit**

```bash
git add src/gameboard.ts src/collisionmanager.ts src/scoreManager.ts src/resultsscreen.ts
git commit -m "feat: robot rival wiring, level-end flow, and results screen

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

### Task 6: Browser verification and tuning pass

**Files:**
- None expected (tuning tweaks allowed in `src/ai/pathfinding.ts` values, `LevelFactory.getLevelConfig` numbers, `RobotPlayer`).

**Interfaces:** n/a — this task plays the game and verifies the phase-1 checklist.

- [ ] **Step 1: Start the dev server**

Run: `npm run dev` in the background; note the local URL (default `http://localhost:5173`).

- [ ] **Step 2: Verify with chrome-devtools MCP**

Open the URL in the browser (chrome-devtools MCP `new_page`/`navigate_page`), then verify each item (screenshots + console):

1. Menu shows "1 Player vs Robot" / "2 Players" / "Start Game" / "How to play"; clicking a mode highlights it.
2. Start in 1-Player mode → countdown → level 1. The magenta snake moves WITHOUT any keyboard input, tracks toward stars/hearts, and dodges tetris blocks (watch for ~20 seconds; take 2–3 screenshots a few seconds apart and compare robot position vs pickups).
3. HUD shows "LEVEL 1 / 3", "PLAYER 1 ... " top, "ROBOT ..." bottom.
4. Force a level end (steer player 1 into a tetris block): ResultsScreen appears with both scores, correct winner text, Retry + Menu buttons (Next Level only if score1 > score2).
5. 2-Player mode still works: WASD moves the magenta snake, no robot behavior.
6. No console errors (`list_console_messages`).

- [ ] **Step 3: Tuning check**

Watch a full level-1 run without touching the keyboard. The robot should survive at least ~30 seconds and collect at least one pickup. If it dies instantly or jitters (rapid direction flapping), adjust: think interval (200 ms), the `slice(0, 4)` target candidates, or mistake chances in `getLevelConfig`. Record what was changed and why.

- [ ] **Step 4: Report**

Report the checklist results with screenshots to the user, including any tuning changes made. Do NOT mark Phase 1 complete until every item above passes.

- [ ] **Step 5: Commit any tuning changes**

```bash
git add -A src/
git commit -m "tune: robot AI and level pacing after browser verification

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```
