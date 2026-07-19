// Generated combo blips. Uses the Web Audio context p5.sound already owns,
// so no audio assets and no extra context to unlock.

const COMBO_BASE_HZ = 523.25; // C5

export function playComboBlip(multiplier: number): void {
  try {
    const ctx = getAudioContext();
    // Browsers keep the context suspended until a user gesture unlocks it.
    if (!ctx || ctx.state !== "running") return;

    // Each combo tier steps up a fifth-ish, so climbing chains rise in pitch.
    const frequency = COMBO_BASE_HZ * Math.pow(1.335, multiplier - 1);
    const start = ctx.currentTime;
    const duration = 0.12;

    const oscillator = ctx.createOscillator();
    oscillator.type = "square";
    oscillator.frequency.setValueAtTime(frequency, start);

    const gain = ctx.createGain();
    gain.gain.setValueAtTime(0.0001, start);
    gain.gain.exponentialRampToValueAtTime(0.12, start + 0.01);
    gain.gain.exponentialRampToValueAtTime(0.0001, start + duration);

    oscillator.connect(gain);
    gain.connect(ctx.destination);
    oscillator.start(start);
    oscillator.stop(start + duration);
  } catch {
    // Never let a combo blip crash the game — missing/suspended audio
    // context, or any Web Audio quirk, is not worth interrupting play for.
  }
}
