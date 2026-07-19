import { describe, it, expect } from "vitest";
import { ComboTracker } from "./combo";

describe("ComboTracker", () => {
  it("starts at 1x", () => {
    expect(new ComboTracker().getMultiplier()).toBe(1);
  });

  it("climbs while pickups stay inside the window", () => {
    const combo = new ComboTracker(3000, 5);
    expect(combo.registerPickup(1000)).toBe(1);
    expect(combo.registerPickup(2000)).toBe(2);
    expect(combo.registerPickup(4000)).toBe(3);
  });

  it("drops back to 1x after a gap", () => {
    const combo = new ComboTracker(3000, 5);
    combo.registerPickup(1000);
    combo.registerPickup(2000);
    expect(combo.registerPickup(9000)).toBe(1);
  });

  it("caps at the maximum multiplier", () => {
    const combo = new ComboTracker(3000, 3);
    combo.registerPickup(0);
    combo.registerPickup(500);
    combo.registerPickup(1000);
    expect(combo.registerPickup(1500)).toBe(3);
  });

  it("resets on demand", () => {
    const combo = new ComboTracker(3000, 5);
    combo.registerPickup(1000);
    combo.registerPickup(2000);
    combo.reset();
    expect(combo.getMultiplier()).toBe(1);
    expect(combo.registerPickup(2500)).toBe(1);
  });
});
