// Run-wide progression state: mode, current level, and best finish-time
// persistence per level. No p5 globals — unit-tested in node.

export type GameMode = "onePlayer" | "twoPlayer";

const noStorage: Pick<Storage, "getItem" | "setItem"> = {
  getItem: () => null,
  setItem: () => {},
};

export class Progress {
  public mode: GameMode = "onePlayer";
  public currentLevel: number = 1;
  private storage: Pick<Storage, "getItem" | "setItem">;

  constructor(storage?: Pick<Storage, "getItem" | "setItem">) {
    this.storage =
      storage ?? (typeof localStorage !== "undefined" ? localStorage : noStorage);
  }

  startRun(mode: GameMode): void {
    this.mode = mode;
    this.currentLevel = 1;
  }

  isLastLevel(): boolean {
    return this.currentLevel >= 3;
  }

  private bestTimeKey(level: number): string {
    return `furious-snake-best-time-L${level}`;
  }

  getBestTime(level: number): number | null {
    const raw = this.storage.getItem(this.bestTimeKey(level));
    if (raw === null) return null;
    const n = Number(raw);
    return Number.isFinite(n) ? n : null;
  }

  recordBestTime(
    level: number,
    timeMs: number
  ): { bestMs: number; isNewBest: boolean } {
    const prev = this.getBestTime(level);
    const isNewBest = prev === null || timeMs < prev;
    if (isNewBest) {
      this.storage.setItem(this.bestTimeKey(level), String(timeMs));
    }
    return { bestMs: isNewBest ? timeMs : (prev as number), isNewBest };
  }
}
