import { Player } from "./player";

export class ScoreManager {
  private scores: Map<number, number>;
  private players: Player[];
  private scoreIncrementInterval: number = 500;
  private lastTickTime: number = 0;

  constructor(players: Player[]) {
    this.scores = new Map();
    this.players = players;
    for (const player of players) {
      this.scores.set(player.getPlayerNumber(), 0);
    }
    this.lastTickTime = millis();
  }

  updateScore(playerNumber: number, points: number): void {
    if (this.scores.has(playerNumber)) {
      const currentScore = this.scores.get(playerNumber) || 0;
      this.scores.set(playerNumber, currentScore + points);
      console.log(
        `Player ${playerNumber}'s score updated: ${this.scores.get(playerNumber)}`
      );
    }
  }

  tickScore(): void {
    const currentTime = millis();
    if (currentTime - this.lastTickTime >= this.scoreIncrementInterval) {
      for (const [playerNumber] of this.scores.entries()) {
        this.updateScore(playerNumber, 1);
      }
      this.lastTickTime = currentTime;
    }
  }

  draw(): void {
    push();
    textSize(24);
    fill("white");
    textAlign(CENTER, CENTER);

    for (const [playerNumber, score] of this.scores.entries()) {
      const player = this.players.find(
        (p) => p.getPlayerNumber() === playerNumber
      );
      if (player) {
        const textContent = `${player.displayName} Score: ${score} | Lives: ${player.lives}`;
        if (playerNumber === 1) {
          text(textContent, width / 2, 50);
        } else if (playerNumber === 2) {
          text(textContent, width / 2, height - 40);
        }
      }
    }

    pop();
  }

  getScore(playerNumber: number): number {
    return this.scores.get(playerNumber) || 0;
  }
}
