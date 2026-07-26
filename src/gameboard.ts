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
