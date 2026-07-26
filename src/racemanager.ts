// Tracks the race: a clock, each racer's progress toward the finish, and the
// winner. Everything except draw() is p5-free so it can be unit-tested.

export type RaceReason = "finish" | "fell-behind" | "no-lives" | "opponent-out";

export interface RacerHud {
  pn: number;
  color: string;
  lives: number;
}

export class RaceManager {
  private startX: number;
  private finishX: number;
  private headX: Map<number, number> = new Map();
  private elapsed: number = 0;
  private _winner: number | null = null;
  private _reason: RaceReason | null = null;

  constructor(playerNumbers: number[], startX: number, finishX: number) {
    this.startX = startX;
    this.finishX = finishX;
    for (const pn of playerNumbers) {
      this.headX.set(pn, startX);
    }
  }

  tick(dtMs: number): void {
    if (this._winner === null) {
      this.elapsed += dtMs;
    }
  }

  setHeadX(pn: number, x: number): void {
    this.headX.set(pn, x);
  }

  progress(pn: number): number {
    const span = this.finishX - this.startX;
    if (span <= 0) return 1;
    const raw = ((this.headX.get(pn) ?? this.startX) - this.startX) / span;
    return Math.max(0, Math.min(1, raw));
  }

  elapsedMs(): number {
    return this.elapsed;
  }

  get isOver(): boolean {
    return this._winner !== null;
  }

  declareWinner(pn: number, reason: RaceReason): void {
    if (this._winner !== null) return;
    this._winner = pn;
    this._reason = reason;
  }

  get winner(): number | null {
    return this._winner;
  }

  get winReason(): RaceReason | null {
    return this._reason;
  }

  // p5 draw: a progress bar with a marker per racer, a finish flag, a clock,
  // and a lives readout. Only this method touches p5.
  draw(racers: RacerHud[], levelNumber: number, levelCount: number): void {
    const barX = 210;
    const barW = width - barX - 60;
    const barY = 30;

    push();
    textFont(customFont);

    fill("#45FF8C");
    textSize(14);
    textAlign(LEFT, CENTER);
    text(`LVL ${levelNumber}/${levelCount}`, 20, barY);

    noStroke();
    rectMode(CORNER);
    fill("#2a2a2a");
    rect(barX, barY - 6, barW, 12, 6);

    fill("#45FF8C");
    rect(barX + barW, barY - 10, 6, 20);

    rectMode(CENTER);
    for (const r of racers) {
      const x = barX + barW * this.progress(r.pn);
      fill(r.color);
      rect(x, barY, 12, 18, 3);
    }

    fill("#ffffff");
    textSize(14);
    textAlign(RIGHT, CENTER);
    text(formatTime(this.elapsed), width - 20, barY);

    // Lives pips per racer (small squares, font-independent).
    rectMode(CORNER);
    let ly = barY + 18;
    for (const r of racers) {
      for (let i = 0; i < Math.max(0, r.lives); i++) {
        fill(r.color);
        rect(20 + i * 14, ly, 10, 10, 2);
      }
      ly += 16;
    }
    pop();
  }
}

export function formatTime(ms: number): string {
  const totalSeconds = ms / 1000;
  const m = Math.floor(totalSeconds / 60);
  const s = Math.floor(totalSeconds % 60);
  const tenths = Math.floor((totalSeconds * 10) % 10);
  return `${m}:${s.toString().padStart(2, "0")}.${tenths}`;
}
