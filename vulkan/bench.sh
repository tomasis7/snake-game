#!/usr/bin/env bash
# Build both Vulkan snake implementations in release mode and compare their FPS.
# Usage: [CPU=x86-64-v2] vulkan/bench.sh [seconds] [runs]   (defaults: 10 seconds, 2 runs per config)
# CPU sets the same target CPU level for both compilers (-march / -C target-cpu).
# Unset = baseline x86-64, where Rust's f32::floor is a libm call but GCC inlines floor.
set -euo pipefail

here="$(cd "$(dirname "$0")" && pwd)"
seconds="${1:-10}"
runs="${2:-2}"
cpu="${CPU:-}"
quad_counts=(0 10000 100000)

cpp_build="$here/cpp/build${cpu:+-$cpu}"
rust_target="$here/rust/target${cpu:+/$cpu}"

echo "Building C++ (Release${cpu:+, -march=$cpu})..." >&2
cmake -S "$here/cpp" -B "$cpp_build" -DCMAKE_BUILD_TYPE=Release \
  ${cpu:+"-DCMAKE_CXX_FLAGS=-march=$cpu"} >/dev/null
cmake --build "$cpp_build" -j >/dev/null
echo "Building Rust (release${cpu:+, target-cpu=$cpu})..." >&2
(cd "$here/rust" && RUSTFLAGS="${cpu:+-C target-cpu=$cpu}" \
  cargo build --release --quiet --target-dir "$rust_target")

cpp="$cpp_build/snake_vk"
rust="$rust_target/release/snake_vk"

results="$(mktemp)"
trap 'rm -f "$results"' EXIT

for quads in "${quad_counts[@]}"; do
  for ((run = 1; run <= runs; run++)); do
    # Alternate which binary goes first so thermal drift doesn't favour one side.
    if ((run % 2)); then order=("$cpp" "$rust"); else order=("$rust" "$cpp"); fi
    for bin in "${order[@]}"; do
      if [[ $bin == "$cpp" ]]; then name=cpp; else name=rust; fi
      echo "  $name quads=$quads run=$run" >&2
      "$bin" --bench --quads "$quads" --seconds "$seconds" 2>/dev/null | tail -n 1 >>"$results"
    done
  done
done

echo "CPU target: ${cpu:-x86-64 (baseline)}"
python3 - "$results" <<'EOF'
import json, sys
from collections import defaultdict

rows = [json.loads(line) for line in open(sys.argv[1]) if line.strip()]
agg = defaultdict(list)
for r in rows:
    agg[(r["quads"], r["impl"])].append(r)

gpu = rows[0]["gpu"] if rows else "?"
mode = rows[0]["present_mode"] if rows else "?"
print(f"\nGPU: {gpu}   present mode: {mode}   runs per config: {len(next(iter(agg.values())))}\n")
print(f"{'quads':>8} | {'impl':<4} | {'avg fps':>9} | {'1% low':>9} | {'avg ms':>7} | {'p99 ms':>7}")
print("-" * 60)
for quads in sorted({q for q, _ in agg}):
    fps = {}
    for impl in ("cpp", "rust"):
        rs = agg.get((quads, impl), [])
        if not rs:
            continue
        mean = lambda k: sum(r[k] for r in rs) / len(rs)
        fps[impl] = mean("avg_fps")
        print(f"{quads:>8} | {impl:<4} | {mean('avg_fps'):>9.1f} | {mean('p1_low_fps'):>9.1f} | "
              f"{mean('avg_ms'):>7.3f} | {mean('p99_ms'):>7.3f}")
    if len(fps) == 2:
        diff = (fps["rust"] / fps["cpp"] - 1) * 100
        print(f"{'':>8}   rust vs cpp: {diff:+.1f}% avg fps")
    print("-" * 60)
EOF
