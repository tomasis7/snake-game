import { describe, it, expect } from "vitest";
import { Effects } from "./effects";

describe("Effects particles", () => {
  it("spawns the requested number of particles", () => {
    const fx = new Effects();
    fx.burst(100, 100, "#ffffff", 12);
    expect(fx.particleCount).toBe(12);
  });

  it("retires particles once their life runs out", () => {
    const fx = new Effects();
    fx.burst(100, 100, "#ffffff", 8);
    for (let i = 0; i < 40; i++) fx.update(50);
    expect(fx.particleCount).toBe(0);
  });

  it("keeps particles alive partway through their life", () => {
    const fx = new Effects();
    fx.burst(100, 100, "#ffffff", 8);
    fx.update(100);
    expect(fx.particleCount).toBe(8);
  });

  it("caps the particle pool so bursts can't grow without bound", () => {
    const fx = new Effects();
    for (let i = 0; i < 40; i++) fx.burst(10, 10, "#ffffff", 20);
    expect(fx.particleCount).toBeLessThanOrEqual(240);
  });

  it("clamps a huge frame step so effects survive a stutter", () => {
    const fx = new Effects();
    fx.burst(10, 10, "#ffffff", 8);
    fx.update(100000);
    expect(fx.particleCount).toBe(8);
  });
});

describe("Effects shake", () => {
  it("has no offset at rest", () => {
    const fx = new Effects();
    expect(fx.shakeOffset()).toEqual({ x: 0, y: 0 });
  });

  it("offsets within the requested intensity while shaking", () => {
    const fx = new Effects();
    fx.shake(10);
    const offset = fx.shakeOffset();
    expect(Math.abs(offset.x)).toBeLessThanOrEqual(10);
    expect(Math.abs(offset.y)).toBeLessThanOrEqual(10);
  });

  it("decays back to rest", () => {
    const fx = new Effects();
    fx.shake(10);
    for (let i = 0; i < 40; i++) fx.update(50);
    expect(fx.shakeOffset()).toEqual({ x: 0, y: 0 });
  });

  it("keeps the stronger shake when two overlap", () => {
    const fx = new Effects();
    fx.shake(4);
    fx.shake(12);
    fx.update(0);
    const offset = fx.shakeOffset();
    expect(Math.abs(offset.x)).toBeLessThanOrEqual(12);
  });
});

describe("Effects flash and floating text", () => {
  it("fades the flash out", () => {
    const fx = new Effects();
    fx.flash("#ffffff");
    expect(fx.flashAlpha).toBeGreaterThan(0);
    for (let i = 0; i < 40; i++) fx.update(50);
    expect(fx.flashAlpha).toBe(0);
  });

  it("retires floating text after its life", () => {
    const fx = new Effects();
    fx.floatText(50, 50, "MINE!", "#ff00ff");
    expect(fx.textCount).toBe(1);
    for (let i = 0; i < 40; i++) fx.update(50);
    expect(fx.textCount).toBe(0);
  });

  it("caps how many floating texts stack up", () => {
    const fx = new Effects();
    for (let i = 0; i < 40; i++) fx.floatText(10, 10, "x2", "#ffffff");
    expect(fx.textCount).toBeLessThanOrEqual(12);
  });
});
