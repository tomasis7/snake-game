// Chains pickups collected in quick succession into a score multiplier.
// Pure: the caller supplies the clock, so this is fully unit-testable.

export class ComboTracker {
  private lastPickupAt: number | null = null;
  private multiplier: number = 1;

  constructor(
    private readonly windowMs: number = 3000,
    private readonly maxMultiplier: number = 5
  ) {}

  registerPickup(now: number): number {
    const inWindow =
      this.lastPickupAt !== null && now - this.lastPickupAt <= this.windowMs;
    this.multiplier = inWindow
      ? Math.min(this.multiplier + 1, this.maxMultiplier)
      : 1;
    this.lastPickupAt = now;
    return this.multiplier;
  }

  reset(): void {
    this.multiplier = 1;
    this.lastPickupAt = null;
  }

  getMultiplier(): number {
    return this.multiplier;
  }
}
