import { describe, it, expect } from "vitest";
import { RaceManager, formatTime } from "./racemanager";

describe("RaceManager progress", () => {
  it("is 0 at the start line and 1 at the finish", () => {
    const rm = new RaceManager([1, 2], 100, 1100);
    expect(rm.progress(1)).toBe(0);
    rm.setHeadX(1, 1100);
    expect(rm.progress(1)).toBe(1);
  });

  it("clamps beyond the finish and before the start", () => {
    const rm = new RaceManager([1], 100, 1100);
    rm.setHeadX(1, 2000);
    expect(rm.progress(1)).toBe(1);
    rm.setHeadX(1, 0);
    expect(rm.progress(1)).toBe(0);
  });

  it("is a fraction in between", () => {
    const rm = new RaceManager([1], 100, 1100);
    rm.setHeadX(1, 600);
    expect(rm.progress(1)).toBeCloseTo(0.5, 5);
  });

  it("guards a zero-width course", () => {
    const rm = new RaceManager([1], 500, 500);
    expect(rm.progress(1)).toBe(1);
  });
});

describe("RaceManager clock and winner", () => {
  it("accumulates elapsed time from ticks", () => {
    const rm = new RaceManager([1], 0, 100);
    rm.tick(16);
    rm.tick(16);
    expect(rm.elapsedMs()).toBe(32);
  });

  it("freezes the clock once a winner is declared", () => {
    const rm = new RaceManager([1, 2], 0, 100);
    rm.tick(100);
    rm.declareWinner(1, "finish");
    rm.tick(100);
    expect(rm.elapsedMs()).toBe(100);
  });

  it("keeps the first winner and reason", () => {
    const rm = new RaceManager([1, 2], 0, 100);
    rm.declareWinner(2, "opponent-out");
    rm.declareWinner(1, "finish");
    expect(rm.winner).toBe(2);
    expect(rm.winReason).toBe("opponent-out");
    expect(rm.isOver).toBe(true);
  });

  it("has no winner initially", () => {
    const rm = new RaceManager([1, 2], 0, 100);
    expect(rm.winner).toBeNull();
    expect(rm.isOver).toBe(false);
  });
});

describe("formatTime", () => {
  it("formats minutes, seconds, and tenths", () => {
    expect(formatTime(0)).toBe("0:00.0");
    expect(formatTime(65400)).toBe("1:05.4");
    expect(formatTime(9900)).toBe("0:09.9");
  });
});
