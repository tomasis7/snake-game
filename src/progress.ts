// Run-wide progression state (mode, current level, cumulative totals, best
// score persistence). No p5 globals — unit-tested in node.

export type GameMode = "onePlayer" | "twoPlayer";

const BEST_KEY = "furious-snake-best-total";

const noStorage: Pick<Storage, "getItem" | "setItem"> = {
  getItem: () => null,
  setItem: () => {},
};

export class Progress {
  public mode: GameMode = "onePlayer";
  public currentLevel: number = 1;
  private levelScores: Map<number, [number, number]> = new Map();
  private storage: Pick<Storage, "getItem" | "setItem">;

  constructor(storage?: Pick<Storage, "getItem" | "setItem">) {
    this.storage =
      storage ?? (typeof localStorage !== "undefined" ? localStorage : noStorage);
  }

  startRun(mode: GameMode): void {
    this.mode = mode;
    this.currentLevel = 1;
    this.levelScores.clear();
  }

  setLevelScores(level: number, score1: number, score2: number): void {
    this.levelScores.set(level, [score1, score2]);
  }

  getTotal(playerNumber: number): number {
    let total = 0;
    for (const [score1, score2] of this.levelScores.values()) {
      total += playerNumber === 1 ? score1 : score2;
    }
    return total;
  }

  isLastLevel(): boolean {
    return this.currentLevel >= 3;
  }

  finishRun(): { best: number; isNewBest: boolean } {
    const total = this.getTotal(1);
    const previous = Number(this.storage.getItem(BEST_KEY) ?? "0");
    const isNewBest = total > previous;
    if (isNewBest) {
      this.storage.setItem(BEST_KEY, String(total));
    }
    return { best: Math.max(total, previous), isNewBest };
  }
}
