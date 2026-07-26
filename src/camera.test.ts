import { describe, it, expect } from "vitest";
import { fitCamera, advanceKillLine } from "./camera";

describe("fitCamera", () => {
  it("centers on the midpoint of the framed points", () => {
    const cam = fitCamera([100, 300], [200, 200], 1200, 800, 0, 0.1, 1);
    expect(cam.centerX).toBe(200);
    expect(cam.centerY).toBe(200);
  });

  it("zooms out to fit a wide gap", () => {
    // Box width 2000 + 0 padding; viewport 1200 -> scale 0.6.
    const cam = fitCamera([0, 2000], [400, 400], 1200, 800, 0, 0.1, 1);
    expect(cam.scale).toBeCloseTo(0.6, 5);
  });

  it("never zooms in past the max scale when the points are close", () => {
    const cam = fitCamera([500, 520], [400, 400], 1200, 800, 0, 0.5, 1);
    expect(cam.scale).toBe(1);
  });

  it("never zooms out past the min scale", () => {
    const cam = fitCamera([0, 100000], [0, 0], 1200, 800, 0, 0.4, 1);
    expect(cam.scale).toBe(0.4);
  });

  it("accounts for padding around the points", () => {
    // Box width 1000 + 2*100 padding = 1200; viewport 1200 -> scale 1 (capped).
    const cam = fitCamera([0, 1000], [400, 400], 1200, 800, 100, 0.1, 1);
    expect(cam.scale).toBeCloseTo(1, 5);
    expect(cam.centerX).toBe(500);
  });
});

describe("advanceKillLine", () => {
  it("trails the leader by the max gap", () => {
    expect(advanceKillLine(0, 1500, 1000)).toBe(500);
  });

  it("never moves backwards", () => {
    expect(advanceKillLine(500, 1200, 1000)).toBe(500);
  });

  it("stays at zero early in the race", () => {
    expect(advanceKillLine(0, 300, 1000)).toBe(0);
  });
});
