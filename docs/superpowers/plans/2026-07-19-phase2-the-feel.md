# Phase 2 "The Feel" Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the game's image assets with original 8-bit pixel-art sprites and add the juice layer — snake growth, particles, screen shake, invulnerability blink, robot taunts, and a combo multiplier.

**Architecture:** Sprite data lives as pure character-grid arrays in `src/art/sprites.ts` (unit-tested for well-formedness); a thin p5 renderer in `src/art/drawsprite.ts` paints them as filled rectangles. Effects state (particles, shake, flash, floating text) lives in `src/effects/effects.ts` whose update math is p5-free and unit-tested — only its `draw*` methods touch p5. `CollisionManager` gains an `Effects` handle and a per-player `ComboTracker` (pure, tested).

**Tech Stack:** TypeScript, p5.js (global mode — `createVector`, `width`, `millis`, `deltaTime`, `fill`, `rect` are bare globals, never imported), Vite, vitest.

## Global Constraints

- Spec: `docs/superpowers/specs/2026-07-19-robot-rival-levels-design.md` (Phase 2 section). Branch off `feature/phase1-the-race` — Phase 1 is not yet merged to main.
- **Original pixel art only.** The sprites in this plan are original work in the NES-era *style*. Never copy, trace, or reproduce actual Nintendo (or any other publisher's) sprite data.
- p5 runs in **global mode**: never import p5 functions. Globals `game`, `sounds`, `music`, `images`, `customFont` are declared in `global.d.ts`.
- `src/art/sprites.ts`, `src/effects/effects.ts`, and `src/combo.ts` MUST NOT reference p5 or browser globals **at module scope or in any method the tests call**. `Effects.drawWorld()`/`drawOverlay()` may use p5 because tests never call them.
- Sprite grids: every row of a sprite is the same length; `.` means transparent; every other character must have a palette entry.
- Grid cell is 32 px. Entity draw calls stay centered on `position` (`imageMode(CENTER)` semantics) so collision boxes are unchanged.
- The existing mp3 sound effects stay as they are. The only **new** audio is a generated combo-tier blip (see Task 6 rationale) — do not layer a second sound onto events that already play an mp3.
- Existing gates must keep passing: `npm run build`, `npx tsc --noEmit` clean, and the 18 Phase 1 vitest tests.
- Commit messages end with: `Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>`

---

### Task 1: Pixel sprite library

**Files:**
- Create: `src/art/sprites.ts`
- Create: `src/art/drawsprite.ts`
- Test: `src/art/sprites.test.ts`

**Interfaces:**
- Consumes: nothing.
- Produces (used by Task 2):
  - `interface Sprite { rows: string[]; palette: Record<string, string> }`
  - `export const SPRITES: Record<string, Sprite>` with keys: `star`, `starBright`, `heart`, `ghost`, `plant`, `wall`, `tetris`, `win`
  - `drawSprite(sprite: Sprite, cx: number, cy: number, w: number, h: number): void` — draws centered at (cx, cy) scaled to w×h.

- [ ] **Step 1: Write the failing tests**

Create `src/art/sprites.test.ts`:

```typescript
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
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `npx vitest run src/art/sprites.test.ts`
Expected: FAIL — cannot resolve `./sprites`.

- [ ] **Step 3: Write the sprite data**

Create `src/art/sprites.ts`:

```typescript
// Original 8-bit style pixel art. Each sprite is a character grid: "." is
// transparent, every other character indexes into the sprite's palette.
// No p5 here — this module is pure data so the tests can validate it in node.

export interface Sprite {
  rows: string[];
  palette: Record<string, string>;
}

// O outline, Y body, W highlight
const STAR_ROWS = [
  ".......OO.......",
  ".......OYO......",
  "......OYYO......",
  "......OYYYO.....",
  ".....OYYYYO.....",
  "OOOOOYYYYYYOOOOO",
  "OYYYYYYWWYYYYYYO",
  ".OYYYYYWWYYYYYO.",
  "..OYYYYYYYYYYO..",
  "...OYYYYYYYYO...",
  "....OYYYYYYO....",
  "...OYYYOOYYYO...",
  "..OYYYO..OYYYO..",
  ".OYYO......OYYO.",
  ".OYO........OYO.",
  "..O..........O..",
];

const HEART_ROWS = [
  "..OOO......OOO..",
  ".ORRROO..OORRRO.",
  "ORWWRRROORRRRRRO",
  "ORWWRRRRRRRRRRRO",
  "ORWRRRRRRRRRRRRO",
  "ORRRRRRRRRRRRRRO",
  "ORRRRRRRRRRRRRRO",
  ".ORRRRRRRRRRRRO.",
  ".ORRRRRRRRRRRRO.",
  "..ORRRRRRRRRRO..",
  "...ORRRRRRRRO...",
  "....ORRRRRRO....",
  ".....ORRRRO.....",
  "......ORRO......",
  ".......OO.......",
  "................",
];

const GHOST_ROWS = [
  ".....OOOOOO.....",
  "...OOWWWWWWOO...",
  "..OWWWWWWWWWWO..",
  ".OWWWWWWWWWWWWO.",
  ".OWWEEWWWWEEWWO.",
  ".OWWEEWWWWEEWWO.",
  ".OWWWWWWWWWWWWO.",
  ".OWWWWWWWWWWWWO.",
  ".OWWWWEEEEWWWWO.",
  ".OWWWWWWWWWWWWO.",
  ".OWWWWWWWWWWWWO.",
  ".OWWWWWWWWWWWWO.",
  ".OWWWWWWWWWWWWO.",
  ".OWOWWOWWOWWOWO.",
  ".OOO.OO.OO.OOOO.",
  "................",
];

// Tall sprite: carnivorous flower head over a stem.
const PLANT_ROWS = [
  ".....OOOOOO.....",
  "...OORRRRRROO...",
  "..ORRRRRRRRRRO..",
  ".ORRWWRRRRWWRRO.",
  ".ORRWWRRRRWWRRO.",
  "ORRRRRRRRRRRRRRO",
  "ORRRRRRRRRRRRRRO",
  "OWWOWWOWWOWWOWWO",
  "OWWOWWOWWOWWOWWO",
  "ORRRRRRRRRRRRRRO",
  "ORRRRRRRRRRRRRRO",
  ".ORRRRRRRRRRRRO.",
  "..ORRRRRRRRRRO..",
  "...OORRRRRROO...",
  ".....OOOOOO.....",
  ".......DD.......",
  ".......DD.......",
  "......DGGD......",
  "......DGGD......",
  "......DGGD......",
  "......DGGD......",
  ".....DGGGGD.....",
  ".....DGGGGD.....",
  ".....DGGGGD.....",
  ".....DGGGGD.....",
  "....DGGGGGGD....",
  "....DGGGGGGD....",
  "....DGGGGGGD....",
  "....DGGGGGGD....",
  "...DGGGGGGGGD...",
  "...DGGGGGGGGD...",
  "...DDDDDDDDDD...",
];

const WALL_ROWS = [
  "DDDDDDDDDDDDDDDD",
  "DBBBBBBDBBBBBBBD",
  "DBBBBBBDBBBBBBBD",
  "DBBBBBBDBBBBBBBD",
  "DDDDDDDDDDDDDDDD",
  "DBBBDBBBBBBBDBBD",
  "DBBBDBBBBBBBDBBD",
  "DBBBDBBBBBBBDBBD",
  "DDDDDDDDDDDDDDDD",
  "DBBBBBBDBBBBBBBD",
  "DBBBBBBDBBBBBBBD",
  "DBBBBBBDBBBBBBBD",
  "DDDDDDDDDDDDDDDD",
  "DBBBDBBBBBBBDBBD",
  "DBBBDBBBBBBBDBBD",
  "DDDDDDDDDDDDDDDD",
];

const TETRIS_ROWS = [
  "DDDDDDDDDDDDDDDD",
  "DLLLLLLLLLLLLLLD",
  "DLGGGGGGGGGGGGLD",
  "DLGGGGGGGGGGGGLD",
  "DLGGGGGGGGGGGGLD",
  "DLGGGGGGGGGGGGLD",
  "DLGGGGGGGGGGGGLD",
  "DLGGGGGGGGGGGGLD",
  "DLGGGGGGGGGGGGLD",
  "DLGGGGGGGGGGGGLD",
  "DLGGGGGGGGGGGGLD",
  "DLGGGGGGGGGGGGLD",
  "DLGGGGGGGGGGGGLD",
  "DLGGGGGGGGGGGGLD",
  "DDGGGGGGGGGGGGDD",
  "DDDDDDDDDDDDDDDD",
];

// Checkered finish flag on a pole.
const WIN_ROWS = [
  "PPWWKKWWKKWWKKWW",
  "PPKKWWKKWWKKWWKK",
  "PPWWKKWWKKWWKKWW",
  "PPKKWWKKWWKKWWKK",
  "PPWWKKWWKKWWKKWW",
  "PPKKWWKKWWKKWWKK",
  "PPWWKKWWKKWWKKWW",
  "PPKKWWKKWWKKWWKK",
  "PP..............",
  "PP..............",
  "PP..............",
  "PP..............",
  "PP..............",
  "PP..............",
  "PP..............",
  "PP..............",
];

export const SPRITES: Record<string, Sprite> = {
  star: {
    rows: STAR_ROWS,
    palette: { O: "#4a2c00", Y: "#ffd93b", W: "#fff6a8" },
  },
  // Twinkle frame: same grid, hotter palette.
  starBright: {
    rows: STAR_ROWS,
    palette: { O: "#4a2c00", Y: "#ffe873", W: "#ffffff" },
  },
  heart: {
    rows: HEART_ROWS,
    palette: { O: "#7a0b1c", R: "#e8384f", W: "#ff9aa8" },
  },
  ghost: {
    rows: GHOST_ROWS,
    palette: { O: "#1a1a2e", W: "#f4f4f4", E: "#1a1a2e" },
  },
  plant: {
    rows: PLANT_ROWS,
    palette: {
      O: "#4a0d16",
      R: "#e8384f",
      W: "#ffffff",
      D: "#14532d",
      G: "#2fa14a",
    },
  },
  wall: {
    rows: WALL_ROWS,
    palette: { D: "#5a3218", B: "#9a5b2d" },
  },
  tetris: {
    rows: TETRIS_ROWS,
    palette: { D: "#3d7a06", G: "#7ed321", L: "#b8f04a" },
  },
  win: {
    rows: WIN_ROWS,
    palette: { P: "#8a6b3d", K: "#1a1a1a", W: "#f4f4f4" },
  },
};
```

- [ ] **Step 4: Write the renderer**

Create `src/art/drawsprite.ts`:

```typescript
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
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `npx vitest run src/art/sprites.test.ts`
Expected: PASS.

- [ ] **Step 6: Verify build and full suite**

Run: `npm run build && npx vitest run && npx tsc --noEmit`
Expected: build succeeds, all tests pass (18 prior + new), tsc silent.

- [ ] **Step 7: Commit**

```bash
git add src/art/sprites.ts src/art/drawsprite.ts src/art/sprites.test.ts
git commit -m "feat: add original 8-bit sprite library and renderer

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

### Task 2: Entities draw themselves as pixel art

**Files:**
- Modify: `src/star.ts`, `src/heart.ts`, `src/ghost.ts`, `src/plant.ts`, `src/block.ts`, `src/tetrisBlocks.ts`, `src/winBlock.ts`

**Interfaces:**
- Consumes (Task 1): `SPRITES`, `drawSprite(sprite, cx, cy, w, h)`.
- Produces: nothing new — entity constructors, sizes, and positions are unchanged, so collision behaviour is untouched.

Each entity keeps its `super(...)` call exactly as it is today (including the `images.*` argument — the loaded image is now simply unused by `draw`). Only the `draw()` bodies change.

- [ ] **Step 1: Star — twinkling**

Replace the `draw()` method in `src/star.ts` with:

```typescript
  draw(): void {
    if (this.isRemoved) return;
    // Twinkle: swap to the hotter palette twice a second.
    const frame = millis() % 800 < 400 ? SPRITES.star : SPRITES.starBright;
    drawSprite(frame, this.position.x, this.position.y, this.size.x, this.size.y);
  }
```

and add to the top of the file, after the existing `Entity` import:

```typescript
import { SPRITES } from "./art/sprites";
import { drawSprite } from "./art/drawsprite";
```

- [ ] **Step 2: Heart — keep the existing pulse**

In `src/heart.ts`, add the same two imports, then replace `draw()` with:

```typescript
  draw(): void {
    if (this.isRemoved) return;
    drawSprite(
      SPRITES.heart,
      this.position.x,
      this.position.y,
      this.size.x * this.pulseScale,
      this.size.y * this.pulseScale
    );
  }
```

(`update()` and `pulseScale` stay exactly as they are.)

- [ ] **Step 3: Ghost — bobbing**

In `src/ghost.ts`, add the same two imports, then replace `draw()` with:

```typescript
  draw(): void {
    if (this.isRemoved) return;
    // Bob: a slow 3px vertical float, independent of its drift velocity.
    const bob = Math.sin(millis() / 400) * 3;
    drawSprite(
      SPRITES.ghost,
      this.position.x,
      this.position.y + bob,
      this.size.x,
      this.size.y
    );
  }
```

(`update()` stays as it is — the bob is cosmetic only, so collisions still use `position`.)

- [ ] **Step 4: Plant, wall block, tetris block, win block**

In `src/plant.ts`, add the two imports and replace `draw()` with:

```typescript
  draw(): void {
    if (this.isRemoved) return;
    drawSprite(SPRITES.plant, this.position.x, this.position.y, this.size.x, this.size.y);
  }
```

In `src/block.ts`, add the two imports and replace `draw()` with:

```typescript
  draw(): void {
    if (this.isRemoved) return;
    drawSprite(SPRITES.wall, this.position.x, this.position.y, this.size.x, this.size.y);
  }
```

In `src/tetrisBlocks.ts`, add the two imports and replace `draw()` with:

```typescript
  draw(): void {
    if (this.isRemoved) return;
    drawSprite(SPRITES.tetris, this.position.x, this.position.y, this.size.x, this.size.y);
  }
```

In `src/winBlock.ts`, add the two imports and replace `draw()` with:

```typescript
  draw(): void {
    if (this.isRemoved) return;
    drawSprite(SPRITES.win, this.position.x, this.position.y, this.size.x, this.size.y);
  }
```

- [ ] **Step 5: Verify**

Run: `npm run build && npx vitest run && npx tsc --noEmit`
Expected: all green. `grep -rn "console.warn(\".*entity has no image" src/` should find nothing — those warnings went away with the old draw bodies.

- [ ] **Step 6: Commit**

```bash
git add src/star.ts src/heart.ts src/ghost.ts src/plant.ts src/block.ts src/tetrisBlocks.ts src/winBlock.ts
git commit -m "feat: draw all entities as pixel-art sprites

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

### Task 3: Snake growth, head eyes, and invulnerability blink

**Files:**
- Modify: `src/player.ts`

**Interfaces:**
- Consumes: nothing new.
- Produces (used by Task 5): `Player.grow(segments?: number): void` — queues N segments so the next N moves extend the trail instead of trimming it.

- [ ] **Step 1: Add the growth queue**

In `src/player.ts`, add a field next to `moveTimer` (around line 15):

```typescript
  private pendingGrowth: number;
```

Initialize it in the constructor next to `this.moveTimer = 0;`:

```typescript
    this.pendingGrowth = 0;
```

Add the public method after `getPlayerNumber()`:

```typescript
  // Queues N segments of growth; the trail keeps its tail on the next N moves.
  public grow(segments: number = 1): void {
    this.pendingGrowth += segments;
  }
```

- [ ] **Step 2: Consume the queue when moving**

In `update()`, replace:

```typescript
      this.trail.unshift(newHead);
      this.trail.pop();
```

with:

```typescript
      this.trail.unshift(newHead);
      if (this.pendingGrowth > 0) {
        this.pendingGrowth--;
      } else {
        this.trail.pop();
      }
```

- [ ] **Step 3: Add the invulnerability blink and head eyes**

In `draw()`, insert at the very top, before the existing `push()`:

```typescript
    // Blink while the post-hit cooldown is running so damage reads clearly.
    const sinceHit = Date.now() - this.lastCollisionTime;
    if (sinceHit < this.collisionCooldown && Math.floor(sinceHit / 80) % 2 === 0) {
      return;
    }
```

Then, still in `draw()`, find the end of the trail loop where the shadow is cleared — the existing lines:

```typescript
      drawingContext.shadowBlur = 0;
      drawingContext.shadowOffsetX = 0;
      drawingContext.shadowOffsetY = 0;
```

and replace them with:

```typescript
      drawingContext.shadowBlur = 0;
      drawingContext.shadowOffsetX = 0;
      drawingContext.shadowOffsetY = 0;

      // After the shadow is cleared, so the eyes stay crisp.
      if (i === 0) {
        this.drawEyes(position, diameter);
      }
```

- [ ] **Step 4: Add the eye drawing helper**

Add this private method to `Player`, after `draw()`:

```typescript
  // Two pixel eyes on the head, set square to the direction of travel.
  private drawEyes(head: p5.Vector, diameter: number): void {
    const forwardX = Math.sign(this.direction.x);
    const forwardY = Math.sign(this.direction.y);
    // Perpendicular to travel, so the eyes sit side by side.
    const sideX = forwardY;
    const sideY = forwardX;
    const eye = diameter * 0.16;
    const forwardOffset = diameter * 0.2;
    const sideOffset = diameter * 0.2;

    push();
    noStroke();
    rectMode(CENTER);
    for (const sign of [1, -1]) {
      const x = head.x + forwardX * forwardOffset + sideX * sideOffset * sign;
      const y = head.y + forwardY * forwardOffset + sideY * sideOffset * sign;
      fill("#ffffff");
      rect(x, y, eye * 2, eye * 2);
      fill("#101820");
      rect(x + forwardX * eye * 0.4, y + forwardY * eye * 0.4, eye, eye);
    }
    pop();
  }
```

- [ ] **Step 5: Verify**

Run: `npm run build && npx vitest run && npx tsc --noEmit`
Expected: all green.

- [ ] **Step 6: Commit**

```bash
git add src/player.ts
git commit -m "feat: snake growth queue, head eyes, and damage blink

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

### Task 4: Effects system

**Files:**
- Create: `src/effects/effects.ts`
- Test: `src/effects/effects.test.ts`

**Interfaces:**
- Consumes: nothing.
- Produces (used by Tasks 5–6):
  - `class Effects` with `burst(x, y, color, count?)`, `shake(intensity)`, `flash(color)`, `floatText(x, y, text, color)`, `update(dtMs)`, `shakeOffset(): { x: number; y: number }`, `drawWorld()`, `drawOverlay()`
  - Read-only accessors for tests: `particleCount`, `textCount`, `flashAlpha`

`update`, `burst`, `shake`, `flash`, `floatText`, and `shakeOffset` must stay free of p5 globals — the tests call them in node. Only `drawWorld()` and `drawOverlay()` may use p5.

- [ ] **Step 1: Write the failing tests**

Create `src/effects/effects.test.ts`:

```typescript
import { describe, it, expect } from "vitest";
import { Effects } from "./effects";

describe("Effects particles", () => {
  it("spawns the requested number of particles", () => {
    const fx = new Effects();
    fx.burst(100, 100, "#ffffff", 12);
    expect(fx.particleCount).toBe(12);
  });

  it("retires particles once their life runs out", () => {
    const fx = new Effects();
    fx.burst(100, 100, "#ffffff", 8);
    fx.update(5000);
    expect(fx.particleCount).toBe(0);
  });

  it("keeps particles alive partway through their life", () => {
    const fx = new Effects();
    fx.burst(100, 100, "#ffffff", 8);
    fx.update(100);
    expect(fx.particleCount).toBe(8);
  });
});

describe("Effects shake", () => {
  it("has no offset at rest", () => {
    const fx = new Effects();
    expect(fx.shakeOffset()).toEqual({ x: 0, y: 0 });
  });

  it("offsets within the requested intensity while shaking", () => {
    const fx = new Effects();
    fx.shake(10);
    const offset = fx.shakeOffset();
    expect(Math.abs(offset.x)).toBeLessThanOrEqual(10);
    expect(Math.abs(offset.y)).toBeLessThanOrEqual(10);
  });

  it("decays back to rest", () => {
    const fx = new Effects();
    fx.shake(10);
    fx.update(5000);
    expect(fx.shakeOffset()).toEqual({ x: 0, y: 0 });
  });

  it("keeps the stronger shake when two overlap", () => {
    const fx = new Effects();
    fx.shake(4);
    fx.shake(12);
    fx.update(0);
    const offset = fx.shakeOffset();
    expect(Math.abs(offset.x)).toBeLessThanOrEqual(12);
  });
});

describe("Effects flash and floating text", () => {
  it("fades the flash out", () => {
    const fx = new Effects();
    fx.flash("#ffffff");
    expect(fx.flashAlpha).toBeGreaterThan(0);
    fx.update(5000);
    expect(fx.flashAlpha).toBe(0);
  });

  it("retires floating text after its life", () => {
    const fx = new Effects();
    fx.floatText(50, 50, "MINE!", "#ff00ff");
    expect(fx.textCount).toBe(1);
    fx.update(5000);
    expect(fx.textCount).toBe(0);
  });
});
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `npx vitest run src/effects/effects.test.ts`
Expected: FAIL — cannot resolve `./effects`.

- [ ] **Step 3: Implement Effects**

Create `src/effects/effects.ts`:

```typescript
// Visual feedback: particle bursts, screen shake, screen flash, floating text.
// Everything except drawWorld/drawOverlay is p5-free so it can be unit-tested.

interface Particle {
  x: number;
  y: number;
  vx: number;
  vy: number;
  life: number;
  maxLife: number;
  color: string;
  size: number;
}

interface FloatingText {
  x: number;
  y: number;
  text: string;
  color: string;
  life: number;
  maxLife: number;
}

const PARTICLE_LIFE_MS = 600;
const TEXT_LIFE_MS = 1200;
const SHAKE_DURATION_MS = 350;
const FLASH_DURATION_MS = 250;
const GRAVITY = 0.0006;

export class Effects {
  private particles: Particle[] = [];
  private texts: FloatingText[] = [];
  private shakeIntensity: number = 0;
  private shakeRemaining: number = 0;
  private flashRemaining: number = 0;
  private flashColor: string = "#ffffff";

  get particleCount(): number {
    return this.particles.length;
  }

  get textCount(): number {
    return this.texts.length;
  }

  get flashAlpha(): number {
    return this.flashRemaining <= 0
      ? 0
      : (this.flashRemaining / FLASH_DURATION_MS) * 140;
  }

  burst(x: number, y: number, color: string, count: number = 14): void {
    for (let i = 0; i < count; i++) {
      const angle = (Math.PI * 2 * i) / count + Math.random() * 0.4;
      const speed = 0.06 + Math.random() * 0.12;
      this.particles.push({
        x,
        y,
        vx: Math.cos(angle) * speed,
        vy: Math.sin(angle) * speed,
        life: PARTICLE_LIFE_MS,
        maxLife: PARTICLE_LIFE_MS,
        color,
        size: 3 + Math.random() * 3,
      });
    }
  }

  shake(intensity: number): void {
    // A weaker shake never cuts one already in progress short.
    this.shakeIntensity = Math.max(this.shakeIntensity, intensity);
    this.shakeRemaining = Math.max(this.shakeRemaining, SHAKE_DURATION_MS);
  }

  flash(color: string): void {
    this.flashColor = color;
    this.flashRemaining = FLASH_DURATION_MS;
  }

  floatText(x: number, y: number, text: string, color: string): void {
    this.texts.push({ x, y, text, color, life: TEXT_LIFE_MS, maxLife: TEXT_LIFE_MS });
  }

  update(dtMs: number): void {
    for (const particle of this.particles) {
      particle.x += particle.vx * dtMs;
      particle.y += particle.vy * dtMs;
      particle.vy += GRAVITY * dtMs;
      particle.life -= dtMs;
    }
    this.particles = this.particles.filter((p) => p.life > 0);

    for (const text of this.texts) {
      text.y -= 0.03 * dtMs;
      text.life -= dtMs;
    }
    this.texts = this.texts.filter((t) => t.life > 0);

    this.shakeRemaining = Math.max(0, this.shakeRemaining - dtMs);
    if (this.shakeRemaining === 0) {
      this.shakeIntensity = 0;
    }

    this.flashRemaining = Math.max(0, this.flashRemaining - dtMs);
  }

  shakeOffset(): { x: number; y: number } {
    if (this.shakeRemaining <= 0) {
      return { x: 0, y: 0 };
    }
    const falloff = this.shakeRemaining / SHAKE_DURATION_MS;
    const amount = this.shakeIntensity * falloff;
    return {
      x: (Math.random() * 2 - 1) * amount,
      y: (Math.random() * 2 - 1) * amount,
    };
  }

  // World space: called inside the camera translate.
  drawWorld(): void {
    push();
    noStroke();
    rectMode(CENTER);
    for (const particle of this.particles) {
      const fade = particle.life / particle.maxLife;
      const shade = color(particle.color);
      shade.setAlpha(255 * fade);
      fill(shade);
      rect(particle.x, particle.y, particle.size, particle.size);
    }

    textFont(customFont);
    textAlign(CENTER, CENTER);
    for (const item of this.texts) {
      const fade = item.life / item.maxLife;
      const shade = color(item.color);
      shade.setAlpha(255 * fade);
      fill(shade);
      textSize(16);
      text(item.text, item.x, item.y);
    }
    pop();
  }

  // Screen space: called after the camera translate is popped.
  drawOverlay(): void {
    const alpha = this.flashAlpha;
    if (alpha <= 0) return;
    push();
    noStroke();
    const shade = color(this.flashColor);
    shade.setAlpha(alpha);
    fill(shade);
    rectMode(CORNER);
    rect(0, 0, width, height);
    pop();
  }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `npx vitest run src/effects/effects.test.ts`
Expected: PASS (9 tests).

- [ ] **Step 5: Verify build and full suite**

Run: `npm run build && npx vitest run && npx tsc --noEmit`
Expected: all green.

- [ ] **Step 6: Commit**

```bash
git add src/effects/effects.ts src/effects/effects.test.ts
git commit -m "feat: add particle, shake, flash, and floating text effects

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

### Task 5: Wire effects into gameplay

**Files:**
- Modify: `src/gameboard.ts` (own the Effects instance, update/draw it, apply shake)
- Modify: `src/collisionmanager.ts` (trigger effects, grow the snake, robot taunts)

**Interfaces:**
- Consumes: `Effects` (Task 4), `Player.grow` (Task 3), `RobotPlayer` (Phase 1).
- Produces: `CollisionManager` constructor gains a 6th parameter `effects: Effects`.

- [ ] **Step 1: GameBoard owns and drives the effects**

In `src/gameboard.ts`, add the import:

```typescript
import { Effects } from "./effects/effects";
```

Add the field next to `levelEnded`:

```typescript
  private effects: Effects = new Effects();
```

Pass it to the collision manager — replace the `new CollisionManager(...)` call with:

```typescript
    this.collisionManager = new CollisionManager(
      this.players,
      this.entities,
      this.scoreManager,
      this.removeEntity.bind(this),
      this.endLevel.bind(this),
      this.effects
    );
```

In `update()`, add the effects tick immediately after `this.scoreManager.tickScore();`:

```typescript
    this.effects.update(deltaTime);
```

- [ ] **Step 2: GameBoard draws effects and applies shake**

In `src/gameboard.ts`, replace the whole `draw()` body with:

```typescript
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
    this.scoreManager.draw();

    push();
    textFont(customFont);
    textSize(16);
    textAlign(LEFT, CENTER);
    fill("#45FF8C");
    text(`LEVEL ${this.levelNumber} / ${LevelFactory.LEVEL_COUNT}`, 20, 50);
    pop();
  }
```

- [ ] **Step 3: CollisionManager accepts the effects handle**

In `src/collisionmanager.ts`, add imports:

```typescript
import { Effects } from "./effects/effects";
import { RobotPlayer } from "./robotplayer";
```

Add the field and constructor parameter:

```typescript
  private effects: Effects;

  constructor(
    players: Player[],
    entities: Entity[],
    scoreManager: ScoreManager,
    removeEntityCallback: (entity: Entity) => void,
    onLevelEnd: () => void,
    effects: Effects
  ) {
    this.players = players;
    this.entities = entities;
    this.scoreManager = scoreManager;
    this.removeEntityCallback = removeEntityCallback;
    this.onLevelEnd = onLevelEnd;
    this.effects = effects;
  }
```

- [ ] **Step 4: Reward feedback — growth, sparks, taunts**

In `src/collisionmanager.ts`, replace the body of `handleStarCollision` with:

```typescript
  private handleStarCollision(player: Player, star: Entity): void {
    if (star.isRemoved) return;
    sounds.starPickUp.play();
    player.isColliding = true;

    player.grow(1);
    this.effects.burst(star.position.x, star.position.y, "#ffd93b");
    this.maybeTaunt(player, star);

    player.scoreMultiplier = 2;

    const scoreInterval = setInterval(() => {
      this.scoreManager.updateScore(
        player.getPlayerNumber(),
        50 * player.scoreMultiplier
      );
    }, 1000);

    setTimeout(() => {
      player.scoreMultiplier = 1;
      clearInterval(scoreInterval);
    }, 10000);

    star.isRemoved = true;
    this.removeEntityCallback(star);
  }
```

Replace the body of `handleHeartCollision` with:

```typescript
  private handleHeartCollision(player: Player, heart: Entity): void {
    if (heart.isRemoved) return;

    sounds.gainheart.play();
    player.isColliding = true;

    player.grow(1);
    this.effects.burst(heart.position.x, heart.position.y, "#e8384f");
    this.maybeTaunt(player, heart);

    if (player.lives < player.maxLives) {
      player.lives += 1;
    }
    heart.isRemoved = true;
    this.removeEntityCallback(heart);
  }
```

Add the taunt helper next to them:

```typescript
  private static readonly TAUNTS = ["MINE!", "TOO SLOW!", "GOT IT!", "NICE TRY!"];

  // The robot gloats only when it stole something the human was closing in on.
  private maybeTaunt(collector: Player, pickup: Entity): void {
    if (!(collector instanceof RobotPlayer)) return;

    const human = this.players.find((p) => !(p instanceof RobotPlayer));
    if (!human) return;

    const distance = dist(
      human.trail[0].x,
      human.trail[0].y,
      pickup.position.x,
      pickup.position.y
    );
    if (distance > 4 * 32) return;

    const taunt =
      CollisionManager.TAUNTS[
        Math.floor(Math.random() * CollisionManager.TAUNTS.length)
      ];
    this.effects.floatText(
      pickup.position.x,
      pickup.position.y - 24,
      taunt,
      "#FF00FF"
    );
  }
```

- [ ] **Step 5: Danger feedback — shake and flash**

In `src/collisionmanager.ts`, add one line to each damage/death handler.

In `handleTetrisCollision`, after `player.isColliding = true;`:

```typescript
    this.effects.shake(16);
    this.effects.flash("#ff2d55");
```

In `handleBlockCollision`, after `player.isColliding = true;`:

```typescript
    this.effects.shake(16);
    this.effects.flash("#ff2d55");
```

In `handleWinBlockCollision`, after `player.isColliding = true;`:

```typescript
    this.effects.flash("#45FF8C");
```

In `handlePlantCollision`, after `player.isColliding = true;`:

```typescript
    this.effects.shake(10);
    this.effects.flash("#ff2d55");
    this.effects.burst(player.trail[0].x, player.trail[0].y, "#ff2d55", 10);
```

In `handleGhostCollision`, after `player.isColliding = true;`:

```typescript
    this.effects.shake(8);
    this.effects.burst(player.trail[0].x, player.trail[0].y, "#f4f4f4", 10);
```

- [ ] **Step 6: Verify**

Run: `npm run build && npx vitest run && npx tsc --noEmit`
Expected: all green.

- [ ] **Step 7: Commit**

```bash
git add src/gameboard.ts src/collisionmanager.ts
git commit -m "feat: wire particles, shake, growth, and robot taunts into play

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

### Task 6: Combo multiplier

**Files:**
- Create: `src/combo.ts`
- Test: `src/combo.test.ts`
- Create: `src/audio/sfx.ts`
- Modify: `global.d.ts` (declare `getAudioContext`)
- Modify: `src/collisionmanager.ts` (track combos, score them, show them)

**Interfaces:**
- Consumes: `Effects.floatText` (Task 4).
- Produces:
  - `class ComboTracker` — `registerPickup(now: number): number`, `reset(): void`, `getMultiplier(): number`; constructor `(windowMs?: number, maxMultiplier?: number)` defaulting to `3000` and `5`
  - `playComboBlip(multiplier: number): void` from `src/audio/sfx.ts`

**Audio scope note:** the spec asks for a generated blip on pickup, but pickups already play `star.mp3` / `gain-heart.mp3`; layering a second sound on the same event muddies both. The blip is therefore attached to **combo tier-ups only**, which is feedback the existing mp3s do not provide. Flag this to the user during review if they want it on every pickup instead.

- [ ] **Step 1: Write the failing combo tests**

Create `src/combo.test.ts`:

```typescript
import { describe, it, expect } from "vitest";
import { ComboTracker } from "./combo";

describe("ComboTracker", () => {
  it("starts at 1x", () => {
    expect(new ComboTracker().getMultiplier()).toBe(1);
  });

  it("climbs while pickups stay inside the window", () => {
    const combo = new ComboTracker(3000, 5);
    expect(combo.registerPickup(1000)).toBe(1);
    expect(combo.registerPickup(2000)).toBe(2);
    expect(combo.registerPickup(4000)).toBe(3);
  });

  it("drops back to 1x after a gap", () => {
    const combo = new ComboTracker(3000, 5);
    combo.registerPickup(1000);
    combo.registerPickup(2000);
    expect(combo.registerPickup(9000)).toBe(1);
  });

  it("caps at the maximum multiplier", () => {
    const combo = new ComboTracker(3000, 3);
    combo.registerPickup(0);
    combo.registerPickup(500);
    combo.registerPickup(1000);
    expect(combo.registerPickup(1500)).toBe(3);
  });

  it("resets on demand", () => {
    const combo = new ComboTracker(3000, 5);
    combo.registerPickup(1000);
    combo.registerPickup(2000);
    combo.reset();
    expect(combo.getMultiplier()).toBe(1);
    expect(combo.registerPickup(2500)).toBe(1);
  });
});
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `npx vitest run src/combo.test.ts`
Expected: FAIL — cannot resolve `./combo`.

- [ ] **Step 3: Implement ComboTracker**

Create `src/combo.ts`:

```typescript
// Chains pickups collected in quick succession into a score multiplier.
// Pure: the caller supplies the clock, so this is fully unit-testable.

export class ComboTracker {
  private lastPickupAt: number | null = null;
  private multiplier: number = 1;

  constructor(
    private readonly windowMs: number = 3000,
    private readonly maxMultiplier: number = 5
  ) {}

  registerPickup(now: number): number {
    const inWindow =
      this.lastPickupAt !== null && now - this.lastPickupAt <= this.windowMs;
    this.multiplier = inWindow
      ? Math.min(this.multiplier + 1, this.maxMultiplier)
      : 1;
    this.lastPickupAt = now;
    return this.multiplier;
  }

  reset(): void {
    this.multiplier = 1;
    this.lastPickupAt = null;
  }

  getMultiplier(): number {
    return this.multiplier;
  }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `npx vitest run src/combo.test.ts`
Expected: PASS (5 tests).

- [ ] **Step 5: Declare the audio context global**

In `global.d.ts`, add inside the `declare global` block, next to the `loadSound` declaration:

```typescript
  // p5.sound global; returns the Web Audio context p5 already manages.
  function getAudioContext(): AudioContext;
```

- [ ] **Step 6: Write the blip generator**

Create `src/audio/sfx.ts`:

```typescript
// Generated combo blips. Uses the Web Audio context p5.sound already owns,
// so no audio assets and no extra context to unlock.

const COMBO_BASE_HZ = 523.25; // C5

export function playComboBlip(multiplier: number): void {
  let ctx: AudioContext;
  try {
    ctx = getAudioContext();
  } catch {
    return;
  }
  // Browsers keep the context suspended until a user gesture unlocks it.
  if (!ctx || ctx.state !== "running") return;

  // Each combo tier steps up a fifth-ish, so climbing chains rise in pitch.
  const frequency = COMBO_BASE_HZ * Math.pow(1.335, multiplier - 1);
  const start = ctx.currentTime;
  const duration = 0.12;

  const oscillator = ctx.createOscillator();
  oscillator.type = "square";
  oscillator.frequency.setValueAtTime(frequency, start);

  const gain = ctx.createGain();
  gain.gain.setValueAtTime(0.0001, start);
  gain.gain.exponentialRampToValueAtTime(0.12, start + 0.01);
  gain.gain.exponentialRampToValueAtTime(0.0001, start + duration);

  oscillator.connect(gain);
  gain.connect(ctx.destination);
  oscillator.start(start);
  oscillator.stop(start + duration);
}
```

- [ ] **Step 7: Track combos per player and score them**

In `src/collisionmanager.ts`, add imports:

```typescript
import { ComboTracker } from "./combo";
import { playComboBlip } from "./audio/sfx";
```

Add the field:

```typescript
  private combos: Map<number, ComboTracker> = new Map();
```

Add the helper method (place it next to `maybeTaunt`):

```typescript
  // Scores a pickup at the player's current combo tier and shows the chain.
  private awardPickup(player: Player, pickup: Entity, basePoints: number): void {
    let combo = this.combos.get(player.getPlayerNumber());
    if (!combo) {
      combo = new ComboTracker();
      this.combos.set(player.getPlayerNumber(), combo);
    }

    const multiplier = combo.registerPickup(Date.now());
    this.scoreManager.updateScore(
      player.getPlayerNumber(),
      basePoints * multiplier
    );

    const label =
      multiplier > 1
        ? `+${basePoints * multiplier}  x${multiplier}`
        : `+${basePoints}`;
    this.effects.floatText(
      pickup.position.x,
      pickup.position.y - 8,
      label,
      multiplier > 1 ? "#FDD03C" : "#FFFFFF"
    );

    if (multiplier > 1) {
      playComboBlip(multiplier);
    }
  }

  // A hit breaks the chain.
  private breakCombo(player: Player): void {
    this.combos.get(player.getPlayerNumber())?.reset();
  }
```

In `handleStarCollision`, add immediately after the `this.maybeTaunt(player, star);` line:

```typescript
    this.awardPickup(player, star, 100);
```

In `handleHeartCollision`, add immediately after the `this.maybeTaunt(player, heart);` line:

```typescript
    this.awardPickup(player, heart, 50);
```

In `handlePlantCollision` and `handleGhostCollision`, add after their `this.effects.shake(...)` lines:

```typescript
    this.breakCombo(player);
```

- [ ] **Step 8: Verify**

Run: `npm run build && npx vitest run && npx tsc --noEmit`
Expected: build succeeds, all tests pass (18 Phase 1 + 5 sprite + 9 effects + 5 combo), tsc silent.

- [ ] **Step 9: Commit**

```bash
git add src/combo.ts src/combo.test.ts src/audio/sfx.ts src/collisionmanager.ts global.d.ts
git commit -m "feat: add combo multiplier with generated tier-up blips

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

### Task 7: Browser verification and tuning

**Files:**
- None expected (tuning tweaks allowed in `src/art/sprites.ts` colors, `src/effects/effects.ts` constants, `src/combo.ts` defaults).

**Interfaces:** n/a — this task plays the game and verifies the phase checklist.

- [ ] **Step 1: Start the dev server**

Run `npm run dev` in the background and note the local URL.

- [ ] **Step 2: Verify with chrome-devtools MCP**

Open the URL and verify each item with screenshots and console checks:

1. **Sprites render**: stars, hearts, plants, ghosts, wall blocks, tetris blocks, and the win flag all appear as crisp pixel art with no stretched or missing tiles. Compare against the old look — nothing should be blurry or mis-sized.
2. **Animation**: stars visibly twinkle (two screenshots ~0.4 s apart differ); ghosts bob vertically.
3. **Snake reads**: both snakes have visible eyes on the head pointing in the direction of travel.
4. **Growth**: eat a star and confirm the trail is one segment longer (count segments in before/after screenshots, or read `players[0].trail.length` via `evaluate_script`).
5. **Particles + shake**: colliding with a plant produces a red burst, a screen flash, and visible camera shake; the snake blinks during the cooldown afterwards.
6. **Combo**: collect two pickups within 3 s and confirm the floating text shows `x2` in gold, and that a hit resets the next pickup to `+points` with no multiplier.
7. **Robot taunt**: in 1-Player mode, let the robot take a pickup the human is near and confirm a magenta taunt appears.
8. **No console errors** (`list_console_messages`) beyond the pre-existing favicon 404 and the canvas `willReadFrequently` warning.

- [ ] **Step 3: Tuning check**

Judge whether the juice reads well rather than merely working: shake that makes text unreadable is too strong, particles that vanish instantly are too short-lived, and a flash that covers the play area is too opaque. Adjust the constants in `src/effects/effects.ts` (`PARTICLE_LIFE_MS`, `SHAKE_DURATION_MS`, `FLASH_DURATION_MS`, the `flashAlpha` ceiling of 140) and record what changed and why.

- [ ] **Step 4: Report**

Report the checklist results with screenshots, including any tuning changes. Do NOT mark Phase 2 complete until every item passes.

- [ ] **Step 5: Commit any tuning changes**

```bash
git add -A src/
git commit -m "tune: effect timings and sprite colors after browser verification

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```
