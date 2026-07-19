import { GameScreen } from "./gamescreen";
import { Entity } from "./entity";
import { Player } from "./player";
import { LevelFactory } from "./levelfactory";
import { CollisionManager } from "./collisionmanager";
import { ScoreManager } from "./scoreManager";
import { Ghost } from "./ghost";
import { Heart } from "./heart";
import { GameMode } from "./progress";

export class GameBoard extends GameScreen {
  private entities: Entity[];
  private players: Player[];
  private levelFactory: LevelFactory;
  private collisionManager: CollisionManager;
  private scoreManager: ScoreManager;

  private cameraOffset: number = 0;
  private scrollSpeed: number = 1.5;
  private levelNumber: number;
  private mode: GameMode;

  constructor(levelNumber: number, mode: GameMode) {
    super();
    this.levelNumber = levelNumber;
    this.mode = mode;
    this.players = [
      new Player(createVector(128, 192), 1, "#00FFFF", "green", {
        UP: UP_ARROW,
        DOWN: DOWN_ARROW,
        RIGHT: RIGHT_ARROW,
        LEFT: LEFT_ARROW,
      }),
      new Player(createVector(128, 576), 2, "#FF00FF", "orange", {
        UP: 87,
        DOWN: 83,
        RIGHT: 68,
        LEFT: 65,
      }),
    ];

    this.levelFactory = new LevelFactory();
    const config = this.levelFactory.getLevelConfig(levelNumber);
    this.scrollSpeed = config.scrollSpeed;
    this.entities = this.levelFactory.createEntitiesForLevel(config.layout);

    this.scoreManager = new ScoreManager(this.players);
    this.collisionManager = new CollisionManager(
      this.players,
      this.entities,
      this.scoreManager,
      this.removeEntity.bind(this)
    );
  }

  addEntity(entity: Entity): void {
    if (!(entity instanceof Heart)) {
      this.entities.push(entity);
      console.log(`Entity added:`, entity);
    } else {
      console.log(`Heart entity not added to prevent duplicates.`);
    }
  }

  removeEntity(entity: Entity): void {
    this.entities = this.entities.filter((e) => e !== entity);
    console.log(`Entity removed:`, entity);
    console.log("Current entities after removal:", this.entities);
  }

  public update(): void {
    this.cameraOffset += this.scrollSpeed;

    for (const player of this.players) {
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
  }
}
