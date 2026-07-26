// Pure camera math for framing both racers. No p5 — unit-tested in node.

export interface Camera {
  scale: number;
  centerX: number;
  centerY: number;
}

// Frames all given points (plus padding) into the viewport, centered on their
// midpoint, with the zoom clamped between minScale and maxScale.
export function fitCamera(
  xs: number[],
  ys: number[],
  viewportW: number,
  viewportH: number,
  padding: number,
  minScale: number,
  maxScale: number
): Camera {
  const minX = Math.min(...xs) - padding;
  const maxX = Math.max(...xs) + padding;
  const minY = Math.min(...ys) - padding;
  const maxY = Math.max(...ys) + padding;
  const boxW = Math.max(1, maxX - minX);
  const boxH = Math.max(1, maxY - minY);
  const scale = Math.max(
    minScale,
    Math.min(maxScale, Math.min(viewportW / boxW, viewportH / boxH))
  );
  return { scale, centerX: (minX + maxX) / 2, centerY: (minY + maxY) / 2 };
}

// The kill line trails the leader by maxGap and only ever advances, so a racer
// that falls behind it is out. Monotonic so the pressure never eases.
export function advanceKillLine(
  prev: number,
  leaderX: number,
  maxGap: number
): number {
  return Math.max(prev, leaderX - maxGap);
}
