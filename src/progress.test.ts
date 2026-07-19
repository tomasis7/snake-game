import { describe, it, expect } from "vitest";
import { Progress } from "./progress";

function fakeStorage(): Pick<Storage, "getItem" | "setItem"> & {
  data: Map<string, string>;
} {
  const data = new Map<string, string>();
  return {
    data,
    getItem: (k: string) => data.get(k) ?? null,
    setItem: (k: string, v: string) => void data.set(k, v),
  };
}

describe("Progress", () => {
  it("sums the latest score per level into run totals", () => {
    const p = new Progress(fakeStorage());
    p.startRun("onePlayer");
    p.setLevelScores(1, 100, 80);
    p.setLevelScores(2, 50, 120);
    expect(p.getTotal(1)).toBe(150);
    expect(p.getTotal(2)).toBe(200);
  });

  it("overwrites a level's scores on retry instead of double-counting", () => {
    const p = new Progress(fakeStorage());
    p.startRun("onePlayer");
    p.setLevelScores(1, 100, 80);
    p.setLevelScores(1, 60, 90);
    expect(p.getTotal(1)).toBe(60);
    expect(p.getTotal(2)).toBe(90);
  });

  it("resets totals and level on startRun", () => {
    const p = new Progress(fakeStorage());
    p.startRun("onePlayer");
    p.setLevelScores(1, 100, 80);
    p.startRun("twoPlayer");
    expect(p.getTotal(1)).toBe(0);
    expect(p.currentLevel).toBe(1);
    expect(p.mode).toBe("twoPlayer");
  });

  it("reports last level at level 3", () => {
    const p = new Progress(fakeStorage());
    p.startRun("onePlayer");
    expect(p.isLastLevel()).toBe(false);
    p.currentLevel = 3;
    expect(p.isLastLevel()).toBe(true);
  });

  it("persists a new best total and detects it", () => {
    const storage = fakeStorage();
    const p = new Progress(storage);
    p.startRun("onePlayer");
    p.setLevelScores(1, 300, 10);
    expect(p.finishRun()).toEqual({ best: 300, isNewBest: true });

    const p2 = new Progress(storage);
    p2.startRun("onePlayer");
    p2.setLevelScores(1, 200, 10);
    expect(p2.finishRun()).toEqual({ best: 300, isNewBest: false });
  });

  it("treats a corrupted stored best as 0 and overwrites it", () => {
    const storage = fakeStorage();
    storage.setItem("furious-snake-best-total", "garbage");
    const p = new Progress(storage);
    p.startRun("onePlayer");
    p.setLevelScores(1, 42, 0);
    expect(p.finishRun()).toEqual({ best: 42, isNewBest: true });
    expect(storage.getItem("furious-snake-best-total")).toBe("42");
  });
});
