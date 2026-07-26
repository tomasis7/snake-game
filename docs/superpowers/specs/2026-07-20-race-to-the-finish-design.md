# Race to the Finish — Design (Genre Pivot)

**Date:** 2026-07-20
**Status:** Approved by user (brainstorming session)
**Supersedes the core loop of:** `2026-07-19-robot-rival-levels-design.md` (Phases 1–2 remain the engine; this re-aims the rules)
**Implementation:** Phased; each phase built by Sonnet 5 subagents, orchestrated and reviewed in the main session.

## Problem

The game became a score-collectathon: grab stars/hearts, highest score wins. The user does not enjoy that. They want an **endless-runner race** — reach the finish on the far right, where items are **speed/survival power-ups** (not points) and the optimal play is the **shortest rightward route**, contested against the robot rival.

## What stays (the engine, already built in Phases 1–2)

Pixel-art sprites and cached-buffer rendering, particles / screen shake / damage flash, snake head eyes, the robot's BFS pathfinding brain, the 3-level speed-ramp progression, robot taunts, the scrolling camera, and the finish (win) block on the far right.

## What is removed

- **Score system** (`ScoreManager`'s point tracking and HUD), the **combo multiplier** (`ComboTracker`, `awardPickup`, blips), and **eat-to-grow** (`Player.grow` growth queue) — all score-shaped mechanics.
- The **"highest score at level end" win rule.**

## Decisions made

| Question | Decision |
|---|---|
| Win a level | **First racer to touch the finish block wins** (head-to-head vs the robot) |
| Challenge | Obstacles block the direct line; falling off the left edge kills; hazards cost a life + stun; scroll speed ramps up |
| Items | All are power-ups: **speed boost, shield/extra-life, phase mode, robot trap** — never points |
| Snake | **Fixed short length**, full 4-direction steering, no growth, trail cosmetic |
| Failure | Fall off the left edge, OR lives reach 0 |
| Block/plant/ghost hit | Costs 1 life + brief stun (slow), NOT instant death |
| HUD | **You-vs-robot progress bar** toward the finish, plus finish time |
| Persistence | Best **finish time** per level in localStorage (replaces best score) |
| Levels | Keep 3, each faster scroll + sharper robot (already built) |

## Core loop

The camera auto-scrolls right at the level's base speed, ramping up the further right you get. You steer a fixed-length snake to weave through obstacles toward the finish block. The robot races you on its shortest-path line. Grab a power-up only when it is on or near your line. **First to touch the finish wins the level;** the loser is whoever finishes second, falls off the left edge, or runs out of lives first.

## Camera & the fall-behind rule (revised 2026-07-20)

The fixed rightward auto-scroll is replaced by a **zoom-to-fit camera** that always frames both racers: it centres on the midpoint of the two heads and zooms out (down to a minimum scale) so both stay on screen, zooming back in when they close up. The forward pressure is now **relative to the leader** via a **kill line** that trails the leader by a fixed gap and only ever advances. A racer whose head falls behind the kill line is eliminated; the other racer wins. As a trailing racer nears the kill line, a flashing **"OUT OF TIME!"** warning appears over it. Because the gap is capped by the kill line, the two snakes can never separate by more than one screen — solving "you can't see both snakes." The robot is fed the kill line as its left-edge reference, so its existing survival brain needs no changes.

## Failure & hazards

- **Fall-behind death:** if a racer's head falls behind the moving kill line, that racer is out. For the human this ends the run as a loss; if the robot falls behind, the human wins. (This replaces the earlier fixed-scroll "left screen edge" death.)
- **Lives:** start at 3. A hazard hit (wall, tetris block, plant, ghost) subtracts 1 life and applies a **stun**: the racer cannot move for ~600 ms and is briefly invulnerable (reuse the existing collision-cooldown blink). Lives at 0 → that racer is out.
- **Hazards no longer instant-kill.** `handleTetrisCollision` / `handleBlockCollision` change from "game over" to "lose a life + stun."
- **Scroll ramp:** base scroll speed comes from the level config; it increases with distance travelled (e.g. `speed = base + k · (cameraOffset / width)`), clamped to a max.

## Power-ups (replace the score items in the level layout)

Level layout tile numbers gain power-up types (extend `LevelFactory.createEntitiesForLevel`'s `ENTITY_MAP`). Each is a timed effect owned by the collecting `Player`:

| Power-up | Source tile | Effect |
|---|---|---|
| **Speed boost** | replaces `star` (2) | +50% forward speed for ~4 s |
| **Shield / life** | `heart` (3) | +1 life if below max, else a shield that absorbs the next hazard hit |
| **Phase mode** | new tile | pass through obstacles for ~3 s (reuse `canPassThroughObstacles`); hazards do nothing while phased |
| **Robot trap** | new tile | robot is stunned ~1 s (human pickup only; no-op if the robot grabs it) |

The robot values speed/phase/shield pickups in its target selection when they sit near its rightward line (extend the existing `decideDirection` pickup weighting); it ignores the trap.

## Win / lose flow

- Level ends the instant either racer touches the finish block, or a racer is eliminated (left-edge / lives-0).
- **Results screen** (replacing score comparison): "YOU REACHED THE GOAL!" / "ROBOT WINS!" / "YOU FELL BEHIND", the human's **finish time**, whether it is a new best, and Next Level (only on a human win) / Retry / Menu.

## HUD

A slim **progress bar** across the top: the level's total width maps to the bar; two markers (cyan = you, magenta = robot) show each racer's `head.x` as a fraction of the distance to the finish block, with a flag icon at the right end. A small running **timer** sits beside it. The per-player score/lives text is replaced by a compact lives pip readout.

## Component impact

- `CollisionManager`: finish-block handler → "first toucher wins"; hazard handlers → life + stun (no game over); star/heart handlers → power-up effects; remove score/combo calls.
- `ScoreManager` → **RaceManager**: tracks each racer's progress fraction, elapsed time, elimination, and the winner; owns the progress-bar + timer draw.
- `Player`: add `applySpeedBoost`, `applyStun`, `applyShield`; remove the growth queue; keep fixed trail length.
- `RobotPlayer`: already beelines right; ensure power-ups are weighted but never override survival.
- `LevelFactory`: new power-up tiles; per-level `scrollSpeed` becomes the *base* for the ramp.
- `Progress`: store best finish time per level instead of best total score.

## Testing

- Pure/unit-tested (vitest): scroll-ramp math, stun timing, power-up expiry, RaceManager progress-fraction and winner logic, left-edge elimination, robot power-up weighting.
- Feel/visual (chrome-devtools MCP): the race reads clearly, the progress bar tracks both racers, power-ups visibly change speed/phase, hazards stun rather than kill, and a full level can be won and lost each way.

## Phasing

1. **Race rules** — win-by-reaching-finish, left-edge death, hazard→life+stun, remove scoring, RaceManager + progress-bar HUD, results screen rewrite.
2. **Power-ups** — speed / shield / phase / trap pickups, robot weighting, scroll ramp.
3. **Polish pass** — tune speeds, power-up durations, and level layouts so the shortest-right line is contestable; keep the Phase-2 juice.

## Out of scope

- Solo/time-trial mode, online play, procedural levels, a level editor.
- Keeping any score or combo mechanic.
