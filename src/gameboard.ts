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
import { Effects } from "./effects/effects";

export class GameBoard extends GameScreen {
  private entities: Entity[];
  private players: Player[];
  private levelFactory: LevelFactory;
  private collisionManager: CollisionManager;
  private scoreManager: ScoreManager;
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

    this.scoreManager = new ScoreManager(this.players);
    this.collisionManager = new CollisionManager(
      this.players,
      this.entities,
      this.scoreManager,
      this.removeEntity.bind(this),
      this.endLevel.bind(this),
      this.effects
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
    game.progress.setLevelScores(this.levelNumber, score1, score2);
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
    this.effects.update(deltaTime);
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
}
