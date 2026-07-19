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
  it("accumulates level scores into run totals", () => {
    const p = new Progress(fakeStorage());
    p.startRun("onePlayer");
    p.addLevelScores(100, 80);
    p.addLevelScores(50, 120);
    expect(p.getTotal(1)).toBe(150);
    expect(p.getTotal(2)).toBe(200);
  });

  it("resets totals and level on startRun", () => {
    const p = new Progress(fakeStorage());
    p.startRun("onePlayer");
    p.addLevelScores(100, 80);
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
    p.addLevelScores(300, 10);
    expect(p.finishRun()).toEqual({ best: 300, isNewBest: true });

    const p2 = new Progress(storage);
    p2.startRun("onePlayer");
    p2.addLevelScores(200, 10);
    expect(p2.finishRun()).toEqual({ best: 300, isNewBest: false });
  });
});
