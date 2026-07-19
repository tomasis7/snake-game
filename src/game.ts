import { GameScreen } from "./gamescreen";
import { StartMenu } from "./startmenu";
import { GameBoard } from "./gameboard";
import { CountDown } from "./countdown";
import { Progress, GameMode } from "./progress";

export class Game {
  private activeScreen: GameScreen[];
  public progress: Progress;

  constructor() {
    this.progress = new Progress();
    this.activeScreen = [new StartMenu()];
  }

  changeScreen(newScreen: GameScreen): void {
    this.activeScreen = [newScreen];
  }

  startRun(mode: GameMode): void {
    this.progress.startRun(mode);
    this.startLevel(1);
  }

  startLevel(levelNumber: number): void {
    this.progress.currentLevel = levelNumber;
    this.changeScreen(
      new CountDown(() => {
        this.changeScreen(new GameBoard(levelNumber, this.progress.mode));
      })
    );
  }

  public update(): void {
    for (const screen of this.activeScreen) {
      screen.update();
    }
  }

  draw(): void {
    for (const screen of this.activeScreen) {
      screen.draw();
    }
  }

  end(): void {}
}
