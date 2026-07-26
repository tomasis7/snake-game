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
