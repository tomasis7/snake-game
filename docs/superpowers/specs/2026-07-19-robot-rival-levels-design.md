# Robot Rival, Level Progression & Fun Overhaul — Design

**Date:** 2026-07-19
**Status:** Approved by user (brainstorming session)
**Implementation:** Phased; each phase implemented by Sonnet 5 subagents, orchestrated and reviewed in the main session.

## Problem

The game (a side-scrolling two-player snake runner built on p5.js) feels boring. The user wants:

1. A robot player that automatically moves the shortest distance to targets.
2. Different levels.
3. A general increase in the joy of playing.

## Decisions made

| Question | Decision |
|---|---|
| Robot role | **Rival racer** — competes for pickups and score |
| Player modes | **Menu choice**: "1 Player" (vs robot) or "2 Players" (current arrows-vs-WASD game) |
| Level system | **Progression**: Level 1 → 2 → 3, each faster with a smarter robot |
| Win rule | **Highest score when the level ends** (win block reached or lives out) |
| Fun pillars | All four: tension & stakes, game juice, meaningful choices, clear goals |
| Visual style | **Retro 8-bit pixel-art icons** (original art in the recognizable NES-era style; no copied Nintendo sprites) |
| Build strategy | **Phased milestones**: Phase 1 Race → Phase 2 Feel → Phase 3 Strategy; each phase playable before the next starts |

## Phase 1 — The Race

### RobotPlayer

New class `RobotPlayer extends Player` in `src/robotplayer.ts`.

- Overrides only the decision step: where `Player.handleInput()` reads the keyboard, `RobotPlayer.think()` sets `nextDirection`. Movement, trail, drawing, lives, and collision handling are inherited unchanged, so the robot is a normal `Player` to `GameBoard`, `CollisionManager`, and `ScoreManager`.
- `think()` runs each move tick (every 200 ms):
  1. **Snapshot the world** as a grid of the visible playfield (~24×11 cells around the camera). Blocked cells: tetris blocks, both snakes' trail segments, top/bottom walls. The world scrolls in pixels; the snapshot converts to grid coordinates (32 px cells).
  2. **Choose a target**: nearest reachable star or heart ahead of the scroll, weighted by value (star > heart). If none visible, target is "straight right" toward the win block.
  3. **BFS** (breadth-first search) from the robot's head to the target over the snapshot grid; take the first step of the shortest path. BFS is chosen over A* deliberately: the grid is ~260 cells, so exact shortest paths cost microseconds and there is no heuristic to get wrong.
  4. **Fallback**: if no path exists, pick any non-lethal adjacent cell, preferring rightward.
- **Difficulty knob**: `mistakeChance` per level. On a "mistake" tick the robot skips thinking and drifts right. Level 1: 0.25, Level 2: 0.10, Level 3: ~0 plus contested targeting (it prefers the pickup the human is heading toward).

### Game modes

`StartMenu` shows two buttons:

- **1 Player** — Player 1 (arrow keys, cyan) vs RobotPlayer (magenta).
- **2 Players** — today's behavior exactly: arrows vs WASD, no robot.

`GameBoard` receives a mode parameter and constructs the player list accordingly.

### Level progression

- `Game.newGame()` stops passing the dummy `[[1]]`; the flow is StartMenu → mode select → Level 1 → results → Level 2 → results → Level 3 → final results.
- `LevelFactory` gains per-level settings next to the existing layout arrays:
  `{ layout, scrollSpeed, robotMistakeChance }` — Level 2 and 3 scroll faster and sharpen the robot without touching layout data.
- HUD shows "Level N of 3" beside the scores during play.

### Results screen

Shown when either snake reaches the win block or a snake runs out of lives:

- Both scores side by side; "YOU WIN" / "ROBOT WINS" (or "PLAYER 1/2 WINS") in large pixel lettering.
- Buttons: **Next Level** (only if the human won the level), **Retry**, **Menu**.
- After Level 3: total across all levels; best total saved in `localStorage`.

## Phase 2 — The Feel

### Pixel-art icons

Hearts, stars, plants, ghosts, and the win block redrawn as 16×16 pixel maps (arrays of palette indices) rendered as filled squares — crisp at any scale, no image assets. Hard pixel edges, bright limited palette, 2 px dark outlines. Simple 2-frame animations: stars twinkle, ghosts bob. Snakes keep their gradient bodies but gain pixel eyes on the head to read direction.

### Juice (priority order)

1. **Eating feels good** — snake grows one segment per star (the trail is currently fixed at 8 segments and never grows), particle burst in the pickup's color, short generated blip sound (p5.sound oscillators; no audio assets).
2. **Danger feels dangerous** — screen shake and flash on collision; invulnerability blink during the existing collision cooldown.
3. **The rivalry talks** — when the robot takes a pickup the human was near (within ~4 cells), a pixel speech bubble taunt ("MINE!", "TOO SLOW!").
4. **Combo multiplier** — pickups within 3 s of the previous one chain ×2, ×3…, shown as floating text; resets on gap or collision.

## Phase 3 — The Strategy

- **Power-up tiles** (new tile numbers in the layout arrays): speed boost, ghost mode (uses the existing, currently-unused `canPassThroughObstacles`), shrink (removes trail segments).
- **Risky paths**: adjust Level 2/3 layouts so high-value star clusters sit inside tight block corridors — a safe route vs a greedy route.
- The robot values power-ups in its target choice too.

## Testing

- `RobotPlayer.think()`, the grid snapshot, target selection, and BFS are pure logic → unit tests (vitest): path correctness, fallback when boxed in, mistake-chance behavior, contested targeting.
- Feel/visual work is verified by playing in the browser (chrome-devtools MCP) at the end of each phase.
- Each phase ends in a playable, tunable build; tuning feedback gates the next phase.

## Out of scope

- Online/multiplayer networking.
- More than 3 levels or a level editor.
- Copying actual Nintendo sprite art (style homage only, original pixel art).
