import { GameScreen } from "./gamescreen";
import { Entity } from "./entity";
import { Player } from "./player";
import { RobotPlayer } from "./robotplayer";
import { LevelFactory } from "./levelfactory";
import { CollisionManager } from "./collisionmanager";
import { RaceManager, RaceReason, RacerHud } from "./racemanager";
import { ResultsScreen } from "./resultsscreen";
import { fitCamera, advanceKillLine, Camera } from "./camera";
import { Ghost } from "./ghost";
import { WinBlock } from "./winBlock";
import { GameMode } from "./progress";
import { Effects } from "./effects/effects";

// How far a racer may trail the leader before being eliminated, how close to
// that line the warning appears, and the camera framing limits.
const MAX_GAP = 1000;
const WARN_MARGIN = 280;
const CAMERA_PADDING = 220;
const MIN_SCALE = 0.5;
const MAX_SCALE = 1.0;

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

  private killLine: number = 0;

  constructor(levelNumber: number, mode: GameMode) {
    super();
    this.levelNumber = levelNumber;
    this.mode = mode;

    this.levelFactory = new LevelFactory();
    const config = this.levelFactory.getLevelConfig(levelNumber);

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

  private leaderX(): number {
    return Math.max(...this.players.map((p) => p.trail[0].x));
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

    // The kill line trails the leader and only advances — falling behind it is
    // the pressure that replaced the old fixed scroll.
    this.killLine = advanceKillLine(this.killLine, this.leaderX(), MAX_GAP);

    for (const player of this.players) {
      if (player instanceof RobotPlayer) {
        // The robot treats the kill line exactly like the old left edge, so its
        // survival brain keeps working with no changes.
        player.setContext({
          entities: this.entities,
          cameraOffset: this.killLine,
          otherTrails: this.players
            .filter((p) => p !== player)
            .map((p) => p.trail),
        });
      }
      player.update();
    }

    for (const player of this.players) {
      if (player.trail[0].x + player.size.x < this.killLine) {
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

  private camera(): Camera {
    return fitCamera(
      this.players.map((p) => p.trail[0].x),
      this.players.map((p) => p.trail[0].y),
      width,
      height,
      CAMERA_PADDING,
      MIN_SCALE,
      MAX_SCALE
    );
  }

  // Flashing warning over any racer near the kill line, so a trailing snake
  // knows it is about to be eliminated.
  private drawWarnings(cam: Camera): void {
    if (millis() % 500 >= 300) return;
    for (const player of this.players) {
      const head = player.trail[0];
      if (head.x >= this.killLine + WARN_MARGIN) continue;
      const sx = (head.x - cam.centerX) * cam.scale + width / 2;
      const sy = (head.y - cam.centerY) * cam.scale + height / 2;
      push();
      textFont(customFont);
      textAlign(CENTER, CENTER);
      textSize(18);
      fill("#ff2d55");
      text("OUT OF TIME!", sx, sy - 40);
      pop();
    }
  }

  draw(): void {
    background(0);
    const cam = this.camera();
    const shake = this.effects.shakeOffset();

    // Screen-space background with a little parallax from the camera centre.
    const bgShift = ((cam.centerX * 0.25) % 1415) - 1415;
    const numBackgrounds = Math.ceil(width / 1415) + 2;
    for (let i = 0; i < numBackgrounds; i++) {
      image(images.background, i * 1415 - bgShift + shake.x, shake.y, 1415, 800);
    }

    push();
    translate(width / 2 + shake.x, height / 2 + shake.y);
    scale(cam.scale);
    translate(-cam.centerX, -cam.centerY);

    for (const entity of this.entities) {
      entity.draw();
    }

    for (const player of this.players) {
      player.draw();
    }

    this.effects.drawWorld();

    pop();

    this.effects.drawOverlay();
    this.drawWarnings(cam);
    this.raceManager.draw(
      this.racerHud(),
      this.levelNumber,
      LevelFactory.LEVEL_COUNT
    );
  }
}
