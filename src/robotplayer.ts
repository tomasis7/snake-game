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
  fallbackDir,
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

    const { world, pickups } = this.buildWorld();
    const head = toGrid(this.trail[0].x, this.trail[0].y);
    const currentDir = {
      dx: Math.sign(this.direction.x),
      dy: Math.sign(this.direction.y),
    };

    // Difficulty knob: on a "mistake" tick the robot stops chasing pickups,
    // but still refuses lethal moves — dumb, not suicidal.
    if (Math.random() < this.mistakeChance) {
      const safe = fallbackDir(world, head, currentDir);
      this.nextDirection = createVector(safe.dx * CELL, safe.dy * CELL);
      return;
    }

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
