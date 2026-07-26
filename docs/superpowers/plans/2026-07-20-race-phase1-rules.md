# Race Pivot — Phase 1 "Race Rules" Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Turn the game from a score-collectathon into a head-to-head race: first racer to reach the finish block wins, falling off the left edge or running out of lives loses, hazards cost a life and stun instead of instantly killing, and all scoring/combo/growth is removed and replaced by a race clock + you-vs-robot progress bar.

**Architecture:** A new p5-free `RaceManager` owns the clock, per-racer progress fraction, and winner. `CollisionManager` stops scoring: hazards call `Player.applyStun` + lose a life, the finish block declares a winner, and lives-at-zero eliminates. `GameBoard` wires the finish/eliminate callbacks, checks the left edge each frame, resolves the race, and draws the HUD. `Progress` stores a best finish time per level instead of a best score. `ScoreManager` and the combo module are deleted.

**Tech Stack:** TypeScript, p5.js (global mode — `createVector`, `width`, `height`, `deltaTime`, `text`, `rect`, `push`/`pop` are bare globals, never imported), Vite, vitest.

## Global Constraints

- Spec: `docs/superpowers/specs/2026-07-20-race-to-the-finish-design.md`.
- Branch: `feature/race-to-the-finish` (already created; the spec is committed there). Do NOT switch branches.
- p5 runs in **global mode**: never import p5 functions. Globals `game`, `sounds`, `music`, `images`, `customFont` are declared in `global.d.ts`.
- `src/racemanager.ts` (except its `draw` method) and `src/progress.ts` MUST NOT reference p5 or browser globals (except the guarded `localStorage` default already in progress.ts) — they are unit-tested in node.
- Grid cell is 32 px. World coordinates are pixels; a racer's head is `player.trail[0]`.
- **No scoring anywhere.** No points, no combo, no eat-to-grow. Snake length is fixed.
- Win rule: **first racer to touch the finish (WinBlock) wins.** A racer whose head goes off the left edge (`head.x + 32 < cameraOffset`) or whose lives reach 0 is eliminated, and the other racer wins.
- Hazards (wall, tetris block, plant, ghost) cost **1 life** and **stun for 600 ms** (frozen + the existing blink), throttled by the existing `collisionCooldown` (1000 ms). They no longer end the level unless lives hit 0.
- Exactly two racers always exist (1P = you + robot, 2P = P1 + P2); player 1 is "you".
- Existing gates must keep passing after every task: `npm run build`, `npx tsc --noEmit` clean, and the vitest suite.
- Commit messages end with: `Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>`

---

### Task 1: RaceManager (clock, progress, winner)

**Files:**
- Create: `src/racemanager.ts`
- Test: `src/racemanager.test.ts`

**Interfaces:**
- Consumes: nothing.
- Produces (used by Tasks 3–4):
  - `type RaceReason = "finish" | "fell-behind" | "no-lives" | "opponent-out"`
  - `interface RacerHud { pn: number; color: string; lives: number }`
  - `class RaceManager` — constructor `(playerNumbers: number[], startX: number, finishX: number)`; methods `tick(dtMs: number)`, `setHeadX(pn: number, x: number)`, `progress(pn: number): number`, `elapsedMs(): number`, getter `isOver: boolean`, `declareWinner(pn: number, reason: RaceReason)`, getters `winner: number | null` and `winReason: RaceReason | null`, `draw(racers: RacerHud[], levelNumber: number, levelCount: number): void`
  - `formatTime(ms: number): string`

- [ ] **Step 1: Write the failing tests**

Create `src/racemanager.test.ts`:

```typescript
import { describe, it, expect } from "vitest";
import { RaceManager, formatTime } from "./racemanager";

describe("RaceManager progress", () => {
  it("is 0 at the start line and 1 at the finish", () => {
    const rm = new RaceManager([1, 2], 100, 1100);
    expect(rm.progress(1)).toBe(0);
    rm.setHeadX(1, 1100);
    expect(rm.progress(1)).toBe(1);
  });

  it("clamps beyond the finish and before the start", () => {
    const rm = new RaceManager([1], 100, 1100);
    rm.setHeadX(1, 2000);
    expect(rm.progress(1)).toBe(1);
    rm.setHeadX(1, 0);
    expect(rm.progress(1)).toBe(0);
  });

  it("is a fraction in between", () => {
    const rm = new RaceManager([1], 100, 1100);
    rm.setHeadX(1, 600);
    expect(rm.progress(1)).toBeCloseTo(0.5, 5);
  });

  it("guards a zero-width course", () => {
    const rm = new RaceManager([1], 500, 500);
    expect(rm.progress(1)).toBe(1);
  });
});

describe("RaceManager clock and winner", () => {
  it("accumulates elapsed time from ticks", () => {
    const rm = new RaceManager([1], 0, 100);
    rm.tick(16);
    rm.tick(16);
    expect(rm.elapsedMs()).toBe(32);
  });

  it("freezes the clock once a winner is declared", () => {
    const rm = new RaceManager([1, 2], 0, 100);
    rm.tick(100);
    rm.declareWinner(1, "finish");
    rm.tick(100);
    expect(rm.elapsedMs()).toBe(100);
  });

  it("keeps the first winner and reason", () => {
    const rm = new RaceManager([1, 2], 0, 100);
    rm.declareWinner(2, "opponent-out");
    rm.declareWinner(1, "finish");
    expect(rm.winner).toBe(2);
    expect(rm.winReason).toBe("opponent-out");
    expect(rm.isOver).toBe(true);
  });

  it("has no winner initially", () => {
    const rm = new RaceManager([1, 2], 0, 100);
    expect(rm.winner).toBeNull();
    expect(rm.isOver).toBe(false);
  });
});

describe("formatTime", () => {
  it("formats minutes, seconds, and tenths", () => {
    expect(formatTime(0)).toBe("0:00.0");
    expect(formatTime(65400)).toBe("1:05.4");
    expect(formatTime(9900)).toBe("0:09.9");
  });
});
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `npx vitest run src/racemanager.test.ts`
Expected: FAIL — cannot resolve `./racemanager`.

- [ ] **Step 3: Implement RaceManager**

Create `src/racemanager.ts`:

```typescript
// Tracks the race: a clock, each racer's progress toward the finish, and the
// winner. Everything except draw() is p5-free so it can be unit-tested.

export type RaceReason = "finish" | "fell-behind" | "no-lives" | "opponent-out";

export interface RacerHud {
  pn: number;
  color: string;
  lives: number;
}

export class RaceManager {
  private startX: number;
  private finishX: number;
  private headX: Map<number, number> = new Map();
  private elapsed: number = 0;
  private _winner: number | null = null;
  private _reason: RaceReason | null = null;

  constructor(playerNumbers: number[], startX: number, finishX: number) {
    this.startX = startX;
    this.finishX = finishX;
    for (const pn of playerNumbers) {
      this.headX.set(pn, startX);
    }
  }

  tick(dtMs: number): void {
    if (this._winner === null) {
      this.elapsed += dtMs;
    }
  }

  setHeadX(pn: number, x: number): void {
    this.headX.set(pn, x);
  }

  progress(pn: number): number {
    const span = this.finishX - this.startX;
    if (span <= 0) return 1;
    const raw = ((this.headX.get(pn) ?? this.startX) - this.startX) / span;
    return Math.max(0, Math.min(1, raw));
  }

  elapsedMs(): number {
    return this.elapsed;
  }

  get isOver(): boolean {
    return this._winner !== null;
  }

  declareWinner(pn: number, reason: RaceReason): void {
    if (this._winner !== null) return;
    this._winner = pn;
    this._reason = reason;
  }

  get winner(): number | null {
    return this._winner;
  }

  get winReason(): RaceReason | null {
    return this._reason;
  }

  // p5 draw: a progress bar with a marker per racer, a finish flag, a clock,
  // and a lives readout. Only this method touches p5.
  draw(racers: RacerHud[], levelNumber: number, levelCount: number): void {
    const barX = 210;
    const barW = width - barX - 60;
    const barY = 30;

    push();
    textFont(customFont);

    fill("#45FF8C");
    textSize(14);
    textAlign(LEFT, CENTER);
    text(`LVL ${levelNumber}/${levelCount}`, 20, barY);

    noStroke();
    rectMode(CORNER);
    fill("#2a2a2a");
    rect(barX, barY - 6, barW, 12, 6);

    fill("#45FF8C");
    rect(barX + barW, barY - 10, 6, 20);

    rectMode(CENTER);
    for (const r of racers) {
      const x = barX + barW * this.progress(r.pn);
      fill(r.color);
      rect(x, barY, 12, 18, 3);
    }

    fill("#ffffff");
    textSize(14);
    textAlign(RIGHT, CENTER);
    text(formatTime(this.elapsed), width - 20, barY);

    // Lives pips per racer (small squares, font-independent).
    rectMode(CORNER);
    let ly = barY + 18;
    for (const r of racers) {
      for (let i = 0; i < Math.max(0, r.lives); i++) {
        fill(r.color);
        rect(20 + i * 14, ly, 10, 10, 2);
      }
      ly += 16;
    }
    pop();
  }
}

export function formatTime(ms: number): string {
  const totalSeconds = ms / 1000;
  const m = Math.floor(totalSeconds / 60);
  const s = Math.floor(totalSeconds % 60);
  const tenths = Math.floor((totalSeconds * 10) % 10);
  return `${m}:${s.toString().padStart(2, "0")}.${tenths}`;
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `npx vitest run src/racemanager.test.ts`
Expected: PASS (11 tests).

- [ ] **Step 5: Verify build and full suite**

Run: `npm run build && npx vitest run && npx tsc --noEmit`
Expected: build succeeds, all tests pass, tsc silent.

- [ ] **Step 6: Commit**

```bash
git add src/racemanager.ts src/racemanager.test.ts
git commit -m "feat: add RaceManager clock, progress, and winner tracking

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

### Task 2: Player — stun and fixed length

**Files:**
- Modify: `src/player.ts`
- Modify: `src/collisionmanager.ts` (delete the two `player.grow(1)` calls only)

**Interfaces:**
- Consumes: nothing new.
- Produces (used by Task 4): `Player.applyStun(durationMs: number): void` — freezes the racer's movement until `Date.now() + durationMs`. The `grow` method and growth queue are removed (fixed length).

- [ ] **Step 1: Add the stun field and method, remove the growth queue**

In `src/player.ts`:

Remove the growth field declaration (line 17):

```typescript
  private pendingGrowth: number;
```

Remove the `grow` method (lines 34–37):

```typescript
  // Queues N segments of growth; the trail keeps its tail on the next N moves.
  public grow(segments: number = 1): void {
    this.pendingGrowth += segments;
  }
```

Add a stun field where `pendingGrowth` was declared:

```typescript
  private stunnedUntil: number = 0;
```

Remove the growth initialization in the constructor (`this.pendingGrowth = 0;`).

Add the stun method (place it where `grow` was):

```typescript
  // Freezes this racer's movement for the given duration (a hazard penalty).
  public applyStun(durationMs: number): void {
    this.stunnedUntil = Date.now() + durationMs;
  }
```

- [ ] **Step 2: Freeze movement while stunned and restore simple trail advance**

In `update()`, add the stun guard immediately after the `!this.isMoving` guard:

```typescript
  update(): void {
    if (!this.isMoving) {
      return;
    }
    if (Date.now() < this.stunnedUntil) {
      return;
    }
```

Replace the growth-aware trail advance:

```typescript
      this.trail.unshift(newHead);
      if (this.pendingGrowth > 0) {
        this.pendingGrowth--;
      } else {
        this.trail.pop();
      }
```

with the fixed-length advance:

```typescript
      this.trail.unshift(newHead);
      this.trail.pop();
```

- [ ] **Step 3: Delete the growth calls in CollisionManager**

In `src/collisionmanager.ts`, delete the line `player.grow(1);` from `handleStarCollision` and the line `player.grow(1);` from `handleHeartCollision` (two deletions). Do not change anything else in this file yet.

- [ ] **Step 4: Verify**

Run: `npm run build && npx vitest run && npx tsc --noEmit`
Expected: all green. `grep -rn "pendingGrowth\|\.grow(" src/` finds nothing.

- [ ] **Step 5: Commit**

```bash
git add src/player.ts src/collisionmanager.ts
git commit -m "feat: fixed-length snake with a stun instead of growth

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

### Task 3: Progress — best finish time per level

**Files:**
- Modify: `src/progress.ts` (add time API alongside the existing score API — additive, non-breaking)
- Test: `src/progress.test.ts` (add time-API tests)

**Interfaces:**
- Consumes: nothing new.
- Produces (used by Task 4): `Progress.getBestTime(level: number): number | null` and `Progress.recordBestTime(level: number, timeMs: number): { bestMs: number; isNewBest: boolean }`. Lower time is better. (Task 4 removes the now-dead score methods.)

- [ ] **Step 1: Write the failing tests**

In `src/progress.test.ts`, add these tests inside the existing top-level `describe` block (the file already defines a `fakeStorage()` helper — reuse it):

```typescript
  it("records a first best time as a new best", () => {
    const s = fakeStorage();
    const p = new Progress(s);
    expect(p.getBestTime(1)).toBeNull();
    expect(p.recordBestTime(1, 42000)).toEqual({ bestMs: 42000, isNewBest: true });
    expect(p.getBestTime(1)).toBe(42000);
  });

  it("keeps the faster time as best", () => {
    const s = fakeStorage();
    const p = new Progress(s);
    p.recordBestTime(1, 42000);
    expect(p.recordBestTime(1, 50000)).toEqual({ bestMs: 42000, isNewBest: false });
    expect(p.recordBestTime(1, 30000)).toEqual({ bestMs: 30000, isNewBest: true });
  });

  it("tracks best time per level independently", () => {
    const s = fakeStorage();
    const p = new Progress(s);
    p.recordBestTime(1, 40000);
    p.recordBestTime(2, 60000);
    expect(p.getBestTime(1)).toBe(40000);
    expect(p.getBestTime(2)).toBe(60000);
  });

  it("treats a corrupted stored time as no best", () => {
    const s = fakeStorage();
    s.setItem("furious-snake-best-time-L1", "garbage");
    const p = new Progress(s);
    expect(p.getBestTime(1)).toBeNull();
    expect(p.recordBestTime(1, 12345)).toEqual({ bestMs: 12345, isNewBest: true });
  });
```

If `fakeStorage()` in this file does not already expose `setItem`, use the existing object it returns — the file's helper is a `Pick<Storage, "getItem" | "setItem">` with a backing map, so `s.setItem(...)` works.

- [ ] **Step 2: Run tests to verify they fail**

Run: `npx vitest run src/progress.test.ts`
Expected: FAIL — `getBestTime`/`recordBestTime` are not functions.

- [ ] **Step 3: Add the time API to Progress**

In `src/progress.ts`, add these methods inside the `Progress` class (leave the existing score methods in place for now):

```typescript
  private bestTimeKey(level: number): string {
    return `furious-snake-best-time-L${level}`;
  }

  getBestTime(level: number): number | null {
    const raw = this.storage.getItem(this.bestTimeKey(level));
    if (raw === null) return null;
    const n = Number(raw);
    return Number.isFinite(n) ? n : null;
  }

  recordBestTime(
    level: number,
    timeMs: number
  ): { bestMs: number; isNewBest: boolean } {
    const prev = this.getBestTime(level);
    const isNewBest = prev === null || timeMs < prev;
    if (isNewBest) {
      this.storage.setItem(this.bestTimeKey(level), String(timeMs));
    }
    return { bestMs: isNewBest ? timeMs : (prev as number), isNewBest };
  }
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `npx vitest run src/progress.test.ts`
Expected: PASS (existing score tests + 4 new time tests).

- [ ] **Step 5: Verify build and full suite**

Run: `npm run build && npx vitest run && npx tsc --noEmit`
Expected: all green.

- [ ] **Step 6: Commit**

```bash
git add src/progress.ts src/progress.test.ts
git commit -m "feat: track best finish time per level in Progress

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

### Task 4: The race spine — win-by-finish, left-edge death, hazard stun, HUD, results

This is the coordinated swap from scoring to racing. Do the steps in order; the build only returns to green at the end.

**Files:**
- Modify: `src/collisionmanager.ts` (hazards → stun + life; finish/eliminate callbacks; strip scoring/combo)
- Modify: `src/gameboard.ts` (RaceManager, left-edge death, resolve race, HUD)
- Modify: `src/resultsscreen.ts` (winner + time + best; no scores)
- Modify: `src/progress.ts` (remove the dead score methods)
- Modify: `src/progress.test.ts` (remove the dead score tests)
- Delete: `src/scoreManager.ts`, `src/combo.ts`, `src/combo.test.ts`, `src/audio/sfx.ts`

**Interfaces:**
- Consumes: `RaceManager`, `RaceReason`, `RacerHud`, `formatTime` (Task 1); `Player.applyStun` (Task 2); `Progress.getBestTime`/`recordBestTime` (Task 3); existing `RobotPlayer`, `WinBlock`, `Effects`.
- Produces:
  - `CollisionManager` constructor `(players: Player[], entities: Entity[], removeEntityCallback: (e: Entity) => void, onFinish: (pn: number) => void, onEliminate: (pn: number) => void, effects: Effects)`
  - `ResultsScreen` constructor `(levelNumber: number, mode: GameMode, winner: number, humanTimeMs: number, best: { bestMs: number; isNewBest: boolean } | null)`

- [ ] **Step 1: Rewrite CollisionManager**

Replace the entire contents of `src/collisionmanager.ts` with:

```typescript
import { Entity } from "./entity";
import { Player } from "./player";
import { Ghost } from "./ghost";
import { TetrisBlock } from "./tetrisBlocks";
import { Block } from "./block";
import { Star } from "./star";
import { Heart } from "./heart";
import { Plant } from "./plant";
import { WinBlock } from "./winBlock";
import { Effects } from "./effects/effects";

export class CollisionManager {
  players: Player[];
  entities: Entity[];
  private removeEntityCallback: (entity: Entity) => void;
  private onFinish: (playerNumber: number) => void;
  private onEliminate: (playerNumber: number) => void;
  private effects: Effects;

  constructor(
    players: Player[],
    entities: Entity[],
    removeEntityCallback: (entity: Entity) => void,
    onFinish: (playerNumber: number) => void,
    onEliminate: (playerNumber: number) => void,
    effects: Effects
  ) {
    this.players = players;
    this.entities = entities;
    this.removeEntityCallback = removeEntityCallback;
    this.onFinish = onFinish;
    this.onEliminate = onEliminate;
    this.effects = effects;
  }

  // A hazard costs one life and stuns, throttled by the collision cooldown so a
  // single overlap can't drain lives every frame. Phase mode (Phase 2) will set
  // canPassThroughObstacles; today it is always false.
  private handleHazard(player: Player): void {
    const now = Date.now();
    if (now - player.lastCollisionTime < player.collisionCooldown) return;
    player.lastCollisionTime = now;

    if (player.canPassThroughObstacles) return;

    sounds.blockCollision.play();
    this.effects.shake(10);
    this.effects.flash("#ff2d55");
    this.effects.burst(player.trail[0].x, player.trail[0].y, "#ff2d55", 10);
    player.applyStun(600);
    player.isColliding = true;
    player.lives -= 1;

    if (player.lives <= 0) {
      player.lives = 0;
      if (music.backgroundMusic.isPlaying()) {
        music.backgroundMusic.stop();
      }
      this.onEliminate(player.getPlayerNumber());
    }
  }

  private handleFinish(player: Player): void {
    sounds.goalline.play();
    this.effects.flash("#45FF8C");
    this.onFinish(player.getPlayerNumber());
  }

  private handleStarCollision(star: Entity): void {
    if (star.isRemoved) return;
    sounds.starPickUp.play();
    this.effects.burst(star.position.x, star.position.y, "#ffd93b");
    star.isRemoved = true;
    this.removeEntityCallback(star);
  }

  private handleHeartCollision(player: Player, heart: Entity): void {
    if (heart.isRemoved) return;
    sounds.gainheart.play();
    this.effects.burst(heart.position.x, heart.position.y, "#e8384f");
    if (player.lives < player.maxLives) {
      player.lives += 1;
    }
    heart.isRemoved = true;
    this.removeEntityCallback(heart);
  }

  private handleGhostProximity(player: Player, ghost: Entity): void {
    const distance = dist(
      player.trail[0].x,
      player.trail[0].y,
      ghost.position.x,
      ghost.position.y
    );

    if (distance < 200) {
      if (!ghost.isSoundPlaying) {
        sounds.ghost.play();
        ghost.isSoundPlaying = true;
      } else {
        sounds.ghost.stop();
        ghost.isSoundPlaying = false;
      }
    }
  }

  checkCollision(): void {
    for (const player of this.players) {
      const head = player.trail[0];
      const headLeft = head.x;
      const headRight = head.x + player.size.x;
      const headTop = head.y;
      const headBottom = head.y + player.size.y;

      let hasCollision = false;

      for (const entity of this.entities) {
        if (entity instanceof Ghost) {
          this.handleGhostProximity(player, entity);
        }

        const entityLeft = entity.position.x;
        const entityRight = entity.position.x + entity.size.x;
        const entityTop = entity.position.y;
        const entityBottom = entity.position.y + entity.size.y;

        const isColliding =
          headRight > entityLeft &&
          headLeft < entityRight &&
          headBottom > entityTop &&
          headTop < entityBottom;

        if (isColliding) {
          hasCollision = true;

          if (!player.isColliding) {
            if (
              entity instanceof TetrisBlock ||
              entity instanceof Block ||
              entity instanceof Plant ||
              entity instanceof Ghost
            ) {
              this.handleHazard(player);
            } else if (entity instanceof Star) {
              this.handleStarCollision(entity);
            } else if (entity instanceof Heart) {
              this.handleHeartCollision(player, entity);
            } else if (entity instanceof WinBlock) {
              this.handleFinish(player);
            }

            break;
          }
        }

        if (!hasCollision) {
          player.isColliding = false;
        }
      }
    }
  }
}
```

(Note: the `showPopupMessage` helper is dropped — it had no callers.)

- [ ] **Step 2: Rewrite GameBoard**

Replace the entire contents of `src/gameboard.ts` with:

```typescript
import { GameScreen } from "./gamescreen";
import { Entity } from "./entity";
import { Player } from "./player";
import { RobotPlayer } from "./robotplayer";
import { LevelFactory } from "./levelfactory";
import { CollisionManager } from "./collisionmanager";
import { RaceManager, RaceReason, RacerHud } from "./racemanager";
import { ResultsScreen } from "./resultsscreen";
import { Ghost } from "./ghost";
import { WinBlock } from "./winBlock";
import { GameMode } from "./progress";
import { Effects } from "./effects/effects";

export class GameBoard extends GameScreen {
  private entities: Entity[];
  private players: Player[];
  private levelFactory: LevelFactory;
  private collisionManager: CollisionManager;
  private raceManager: RaceManager;
  private levelNumber: number;
  private mode: GameMode;
  private levelEnded: boolean = false;
  private effects: Effects = new Effects();

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

    const winBlock = this.entities.find((e) => e instanceof WinBlock);
    const finishX = winBlock ? winBlock.position.x : width * 4;
    const startX = playerOne.trail[0].x;
    this.raceManager = new RaceManager([1, 2], startX, finishX);

    this.collisionManager = new CollisionManager(
      this.players,
      this.entities,
      this.removeEntity.bind(this),
      this.onFinish.bind(this),
      this.onEliminate.bind(this),
      this.effects
    );
  }

  removeEntity(entity: Entity): void {
    this.entities = this.entities.filter((e) => e !== entity);
  }

  private other(playerNumber: number): number {
    return playerNumber === 1 ? 2 : 1;
  }

  private onFinish(playerNumber: number): void {
    this.resolveRace(playerNumber, "finish");
  }

  private onEliminate(playerNumber: number): void {
    this.resolveRace(this.other(playerNumber), "opponent-out");
  }

  private resolveRace(winnerPlayerNumber: number, reason: RaceReason): void {
    if (this.levelEnded) return;
    this.levelEnded = true;
    this.raceManager.declareWinner(winnerPlayerNumber, reason);

    const humanTimeMs = this.raceManager.elapsedMs();
    // A best time only counts when the human actually crossed the finish line.
    const best =
      winnerPlayerNumber === 1 && reason === "finish"
        ? game.progress.recordBestTime(this.levelNumber, humanTimeMs)
        : null;

    game.changeScreen(
      new ResultsScreen(
        this.levelNumber,
        this.mode,
        winnerPlayerNumber,
        humanTimeMs,
        best
      )
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

    // Left-edge death: a racer whose head scrolls off the left edge is out.
    for (const player of this.players) {
      if (player.trail[0].x + player.size.x < this.cameraOffset) {
        this.resolveRace(this.other(player.getPlayerNumber()), "fell-behind");
        if (this.levelEnded) return;
      }
    }

    for (const entity of this.entities) {
      entity.update();
    }

    this.flyingGhost();

    this.collisionManager.checkCollision();

    for (const player of this.players) {
      this.raceManager.setHeadX(player.getPlayerNumber(), player.trail[0].x);
    }
    this.raceManager.tick(deltaTime);
    this.effects.update(deltaTime);
  }

  private flyingGhost(): void {
    for (const entity of this.entities) {
      if (entity instanceof Ghost) {
        entity.update();
      }
    }
  }

  private racerHud(): RacerHud[] {
    return this.players.map((p) => ({
      pn: p.getPlayerNumber(),
      color: p.getPlayerNumber() === 1 ? "#00FFFF" : "#FF00FF",
      lives: p.lives,
    }));
  }

  draw(): void {
    background(0);
    const shake = this.effects.shakeOffset();

    const numBackgrounds = Math.ceil((width + this.cameraOffset) / 1415) + 1;
    for (let i = 0; i < numBackgrounds; i++) {
      image(
        images.background,
        i * 1415 - this.cameraOffset + shake.x,
        shake.y,
        1415,
        800
      );
    }
    push();
    translate(-this.cameraOffset + shake.x, shake.y);

    for (const entity of this.entities) {
      entity.draw();
    }

    for (const player of this.players) {
      player.draw();
    }

    this.effects.drawWorld();

    pop();

    this.effects.drawOverlay();
    this.raceManager.draw(
      this.racerHud(),
      this.levelNumber,
      LevelFactory.LEVEL_COUNT
    );
  }
}
```

- [ ] **Step 3: Rewrite ResultsScreen**

Replace the entire contents of `src/resultsscreen.ts` with:

```typescript
import { GameScreen } from "./gamescreen";
import { Button } from "./button";
import { StartMenu } from "./startmenu";
import { GameMode } from "./progress";
import { formatTime } from "./racemanager";

export class ResultsScreen extends GameScreen {
  private levelNumber: number;
  private mode: GameMode;
  private winner: number;
  private humanTimeMs: number;
  private best: { bestMs: number; isNewBest: boolean } | null;
  private isFinal: boolean;

  private nextLevelButton: Button | null = null;
  private retryButton: Button;
  private menuButton: Button;

  constructor(
    levelNumber: number,
    mode: GameMode,
    winner: number,
    humanTimeMs: number,
    best: { bestMs: number; isNewBest: boolean } | null
  ) {
    super();
    this.levelNumber = levelNumber;
    this.mode = mode;
    this.winner = winner;
    this.humanTimeMs = humanTimeMs;
    this.best = best;
    this.isFinal = levelNumber >= 3;

    const humanWon = mode === "onePlayer" ? winner === 1 : winner !== 0;
    if (!this.isFinal && humanWon) {
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
    if (this.mode === "onePlayer") {
      return this.winner === 1 ? "YOU REACHED THE GOAL!" : "ROBOT WINS!";
    }
    return `PLAYER ${this.winner} WINS!`;
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
    textSize(44);
    text(this.winnerText(), width / 2, height / 6 + 90);

    fill("#00FFFF");
    textSize(24);
    text(`TIME  ${formatTime(this.humanTimeMs)}`, width / 2, height / 2 - 20);

    if (this.best) {
      fill("#FDD03C");
      textSize(20);
      text(
        this.best.isNewBest
          ? `NEW BEST!  ${formatTime(this.best.bestMs)}`
          : `BEST  ${formatTime(this.best.bestMs)}`,
        width / 2,
        height / 2 + 20
      );
    }

    if (this.nextLevelButton) this.nextLevelButton.draw();
    this.retryButton.draw();
    this.menuButton.draw();
    pop();
  }
}
```

- [ ] **Step 4: Remove the dead score API from Progress**

In `src/progress.ts`, remove the score-only members: the `levelScores` field, the `BEST_KEY` constant, the `noStorage`... keep `noStorage` (still used by the constructor). Remove `setLevelScores`, `getTotal`, and `finishRun`, and remove `BEST_KEY`. Update the `startRun` method to drop the `this.levelScores.clear();` line (keep the mode/level resets). Keep `getBestTime`, `recordBestTime`, `isLastLevel`, `mode`, `currentLevel`, and the constructor.

In `src/progress.test.ts`, remove the tests that exercise `setLevelScores`, `getTotal`, or `finishRun` (the score API). Keep the time-API tests and any `startRun`/`isLastLevel`/mode tests that don't reference the removed methods.

- [ ] **Step 5: Delete ScoreManager and the combo module**

```bash
git rm src/scoreManager.ts src/combo.ts src/combo.test.ts src/audio/sfx.ts
```

Confirm nothing references them:

```bash
grep -rn "scoreManager\|ScoreManager\|ComboTracker\|playComboBlip\|combo" src/
```

Expected: no matches (the `getAudioContext` declaration left in `global.d.ts` is harmless and may stay).

- [ ] **Step 6: Verify build, types, and full suite**

Run: `npm run build && npx tsc --noEmit && npx vitest run`
Expected: build succeeds, tsc silent, all remaining tests pass. `grep -rn "updateScore\|getScore\|tickScore" src/` finds nothing.

- [ ] **Step 7: Commit**

```bash
git add -A src/
git commit -m "feat: race spine — finish wins, left-edge death, hazard stun, race HUD

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

### Task 5: Browser verification and tuning

**Files:**
- None expected (tuning tweaks allowed in `src/racemanager.ts` bar layout, the `600` stun and `1000` cooldown in `player.ts`/`collisionmanager.ts`, and the left-edge margin in `gameboard.ts`).

**Interfaces:** n/a — this task plays the game and verifies the phase checklist.

- [ ] **Step 1: Start the dev server**

Run `npm run dev` in the background; note the local URL.

- [ ] **Step 2: Verify with chrome-devtools MCP**

Open the URL and verify each item with screenshots, console, and `evaluate_script` reads of `game.activeScreen[0]`:

1. **Race HUD**: a progress bar shows across the top with two markers (cyan = you, magenta = robot) and a finish flag; a running clock ticks; lives pips show. No score text anywhere.
2. **Win by finishing**: steer player 1 into the far-right finish block → ResultsScreen shows "YOU REACHED THE GOAL!", a TIME, and a best-time line; Next Level appears.
3. **Robot can win**: in 1-Player mode, let the robot reach the finish first (don't move) → "ROBOT WINS!", no Next Level.
4. **Left-edge death**: hold still until the scroll carries player 1 off the left edge → the race ends with the robot winning; confirm via `evaluate_script` that the screen became `ResultsScreen`.
5. **Hazard stun, not death**: drive player 1 into a tetris block with >1 life → confirm `players[0].lives` drops by exactly 1, the snake freezes briefly (stun) and blinks, and the game does NOT end; repeat into blocks until lives reach 0 → then it ends.
6. **Fixed length**: eat a star and a heart → confirm `players[0].trail.length` stays 8 (no growth); heart still adds a life.
7. **No console errors** beyond the pre-existing favicon 404 and the canvas `willReadFrequently` warning.

- [ ] **Step 3: Tuning check**

Judge the feel: the scroll speed vs. the snake's 200 ms move cadence should make left-edge pressure real but fair; the stun should read as a stumble, not a lockup; the progress bar should make "who's ahead" obvious at a glance. Adjust the stun duration (`600`), hazard cooldown (`1000`), left-edge margin, or bar layout as needed and record what changed and why.

- [ ] **Step 4: Report**

Report the checklist results with screenshots, including any tuning changes. Do NOT mark Phase 1 complete until every item passes.

- [ ] **Step 5: Commit any tuning changes**

```bash
git add -A src/
git commit -m "tune: race pacing and HUD after browser verification

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```
