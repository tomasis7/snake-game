import { GameScreen } from "./gamescreen";

export class CountDown extends GameScreen {
  private countdownValue: number;
  private lastUpdateTime: number;
  private readonly countdownDuration: number = 3;
  private isComplete: boolean;
  private callback: () => void;

  constructor(callback: () => void) {
    super();
    this.countdownValue = this.countdownDuration;
    this.lastUpdateTime = Date.now();
    this.isComplete = false;
    this.callback = callback;
  }

  update(): void {
    const currentTime = Date.now();
    const deltaSeconds = (currentTime - this.lastUpdateTime) / 1000;
    this.lastUpdateTime = currentTime;

    if (!this.isComplete && this.countdownValue > 0) {
      this.countdownValue -= deltaSeconds;
      if (this.countdownValue <= 0) {
        this.countdownValue = 0;
        this.isComplete = true;
        this.callback();
      }
    }
  }

  draw(): void {
    push();
    background("black");
    fill("#45FF8C");
    textSize(32);
    textAlign(CENTER, CENTER);
    textFont(customFont);
    text("GET READY", width / 2, height / 4);

    fill("#FFFFFF");
    textSize(84);

    const displayNumber = Math.ceil(this.countdownValue);
    if (displayNumber > 0) {
      text(displayNumber.toString(), width / 2, height / 3);
    }
    pop();
  }

  isCountdownComplete(): boolean {
    return this.isComplete;
  }
}
