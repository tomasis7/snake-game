import { GameScreen } from "./gamescreen";
import { Button } from "./button";
import { StartMenu } from "./startmenu";
import { GameMode } from "./progress";

export class ResultsScreen extends GameScreen {
  private levelNumber: number;
  private mode: GameMode;
  private score1: number;
  private score2: number;
  private winner: number; // 1, 2, or 0 for tie
  private isFinal: boolean;
  private finalBest: { best: number; isNewBest: boolean } | null = null;

  private nextLevelButton: Button | null = null;
  private retryButton: Button;
  private menuButton: Button;

  constructor(
    levelNumber: number,
    mode: GameMode,
    score1: number,
    score2: number
  ) {
    super();
    this.levelNumber = levelNumber;
    this.mode = mode;
    this.score1 = score1;
    this.score2 = score2;
    this.winner = score1 > score2 ? 1 : score2 > score1 ? 2 : 0;
    this.isFinal = levelNumber >= 3;

    if (this.isFinal) {
      this.finalBest = game.progress.finishRun();
    }

    const humanAdvances =
      this.mode === "twoPlayer" ? this.winner !== 0 : this.winner === 1;
    if (!this.isFinal && humanAdvances) {
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
    if (this.winner === 0) return "IT'S A TIE!";
    if (this.mode === "onePlayer") {
      return this.winner === 1 ? "YOU WIN!" : "ROBOT WINS!";
    }
    return `PLAYER ${this.winner} WINS!`;
  }

  private nameFor(playerNumber: number): string {
    if (this.mode === "onePlayer") {
      return playerNumber === 1 ? "YOU" : "ROBOT";
    }
    return `PLAYER ${playerNumber}`;
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
    textSize(48);
    text(this.winnerText(), width / 2, height / 6 + 90);

    textSize(22);
    fill("#00FFFF");
    text(
      `${this.nameFor(1)}  ${this.score1}`,
      width / 2 - 220,
      height / 2 - 40
    );
    fill("#FF00FF");
    text(
      `${this.nameFor(2)}  ${this.score2}`,
      width / 2 + 220,
      height / 2 - 40
    );

    if (this.isFinal && this.finalBest) {
      fill("white");
      textSize(18);
      text(
        `RUN TOTAL  ${this.nameFor(1)} ${game.progress.getTotal(1)}  -  ${this.nameFor(2)} ${game.progress.getTotal(2)}`,
        width / 2,
        height / 2 + 30
      );
      fill("#FDD03C");
      text(
        this.finalBest.isNewBest
          ? `NEW BEST: ${this.finalBest.best}!`
          : `BEST: ${this.finalBest.best}`,
        width / 2,
        height / 2 + 70
      );
    }

    if (this.nextLevelButton) this.nextLevelButton.draw();
    this.retryButton.draw();
    this.menuButton.draw();
    pop();
  }
}
