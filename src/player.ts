import { Entity } from "./entity";

export interface KeyBindings {
  UP: number;
  DOWN: number;
  RIGHT: number;
  LEFT: number;
}

export class Player extends Entity {
  public trail: p5.Vector[];
  public playerNumber: number;
  public displayName: string;
  private trailFillColor: string;
  private trailStrokeColor: string;
  private moveTimer: number;
  private pendingGrowth: number;
  protected nextDirection: p5.Vector;
  private keyBindings: KeyBindings;

  public lives: number;
  public maxLives: number;
  public lastCollisionTime: number;
  public collisionCooldown: number;
  public canPassThroughObstacles: boolean;

  public isMoving: boolean;
  public isColliding: boolean = false;

  getPlayerNumber(): number {
    return this.playerNumber;
  }

  // Queues N segments of growth; the trail keeps its tail on the next N moves.
  public grow(segments: number = 1): void {
    this.pendingGrowth += segments;
  }

  constructor(
    position: p5.Vector,
    playerNumber: number,
    trailFillColor: string,
    trailStrokeColor: string,
    keyBindings: KeyBindings
  ) {
    const size = createVector(32, 32);
    position.x = position.x + 16;
    position.y = position.y + 16;
    super(position, size, 0, 0, createVector(0, 500), undefined);
    this.trail = [
      createVector(this.position.x - size.x, this.position.y),
      createVector(this.position.x - size.x * 2, this.position.y),
      createVector(this.position.x - size.x * 3, this.position.y),
      createVector(this.position.x - size.x * 4, this.position.y),
      createVector(this.position.x - size.x * 5, this.position.y),
      createVector(this.position.x - size.x * 6, this.position.y),
      createVector(this.position.x - size.x * 7, this.position.y),
      createVector(this.position.x - size.x * 8, this.position.y),
    ];
    this.playerNumber = playerNumber;
    this.displayName = `PLAYER ${playerNumber}`;
    this.trailFillColor = trailFillColor;
    this.trailStrokeColor = trailStrokeColor;
    this.moveTimer = 0;
    this.pendingGrowth = 0;
    this.direction = createVector(32, 0);
    this.nextDirection = this.direction.copy();
    this.keyBindings = keyBindings;

    this.isMoving = true;

    this.lives = 3;
    this.maxLives = 10;
    this.lastCollisionTime = 0;
    this.collisionCooldown = 1000;
    this.canPassThroughObstacles = false;
  }

  protected handleInput(): void {
    if (keyIsDown(this.keyBindings.UP) && this.direction.y === 0) {
      this.nextDirection = createVector(0, -32);
    } else if (keyIsDown(this.keyBindings.DOWN) && this.direction.y === 0) {
      this.nextDirection = createVector(0, 32);
    } else if (keyIsDown(this.keyBindings.LEFT) && this.direction.x === 0) {
      this.nextDirection = createVector(-32, 0);
    } else if (keyIsDown(this.keyBindings.RIGHT) && this.direction.x === 0) {
      this.nextDirection = createVector(32, 0);
    }
  }

  update(): void {
    if (!this.isMoving) {
      return;
    }
    this.moveTimer += deltaTime;
    if (this.moveTimer >= 200) {
      this.moveTimer = -100;

      this.direction = this.nextDirection.copy();
      const head = this.trail[0];
      const newHead = createVector(
        head.x + this.direction.x,
        head.y + this.direction.y
      );
      this.trail.unshift(newHead);
      if (this.pendingGrowth > 0) {
        this.pendingGrowth--;
      } else {
        this.trail.pop();
      }
    }

    this.handleInput();
  }

  draw(): void {
    // Blink while the post-hit cooldown is running so damage reads clearly.
    const sinceHit = Date.now() - this.lastCollisionTime;
    if (sinceHit < this.collisionCooldown && Math.floor(sinceHit / 80) % 2 === 0) {
      return;
    }

    push();
    strokeWeight(0);

    for (let i = 0; i < this.trail.length; i++) {
      const position = this.trail[i];
      let diameter = Math.max(this.size.x, this.size.y);

      drawingContext.shadowBlur = 15;
      drawingContext.shadowColor = "rgba(0, 0, 0, 0.3)";
      drawingContext.shadowOffsetX = 5;
      drawingContext.shadowOffsetY = 5;

      let ctx = drawingContext;
      let gradient = ctx.createRadialGradient(
        position.x - diameter * 0.3,
        position.y - diameter * 0.3,
        diameter * 0.1,
        position.x,
        position.y,
        diameter * 0.8
      );

      if (i === 0) {
        gradient.addColorStop(0, "#FFE5CC");
        gradient.addColorStop(0.3, "#FFA500");
        gradient.addColorStop(1, "#804600");
      } else {
        gradient.addColorStop(
          0,
          lerpColor(
            color(this.trailFillColor),
            color("#ffffff"),
            0.2
          ).toString()
        );
        gradient.addColorStop(0.3, this.trailFillColor);
        gradient.addColorStop(
          1,
          lerpColor(color(this.trailStrokeColor), color(0), 0.7).toString()
        );
      }

      ctx.fillStyle = gradient;
      noStroke();
      ellipse(position.x, position.y, diameter, diameter);

      drawingContext.shadowBlur = 0;
      drawingContext.shadowOffsetX = 0;
      drawingContext.shadowOffsetY = 0;

      // After the shadow is cleared, so the eyes stay crisp.
      if (i === 0) {
        this.drawEyes(position, diameter);
      }
    }

    pop();
  }

  // Two pixel eyes on the head, set square to the direction of travel.
  private drawEyes(head: p5.Vector, diameter: number): void {
    const forwardX = Math.sign(this.direction.x);
    const forwardY = Math.sign(this.direction.y);
    // Perpendicular to travel, so the eyes sit side by side.
    const sideX = forwardY;
    const sideY = forwardX;
    const eye = diameter * 0.13;
    const forwardOffset = diameter * 0.18;
    const sideOffset = diameter * 0.18;

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

  public enableObstaclePassing(duration: number): void {
    this.canPassThroughObstacles = true;
    console.log(`Player ${this.playerNumber} can now pass through obstacles`);

    setTimeout(() => {
      this.canPassThroughObstacles = false;
      console.log(
        `Player ${this.playerNumber} can no longer pass through obstacles`
      );
    }, duration);
  }

  doubleLives(): void {
    this.lives = Math.min(this.lives * 2, this.maxLives);
    console.log(`Player ${this.playerNumber} now has ${this.lives} lives.`);
  }
}
