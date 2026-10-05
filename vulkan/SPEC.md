# Vulkan Snake — shared spec (C++ and Rust)

Two implementations of the same game, written directly against Vulkan, so their
frame rates can be compared fairly. Anything listed here must behave the same in both.

## Layout
```
vulkan/
  SPEC.md            this file
  shaders/quad.vert  shared GLSL, compiled to SPIR-V by each build with glslc
  shaders/quad.frag
  cpp/               C++20 + Vulkan + GLFW, CMake
  rust/              Rust + ash, cargo
  bench.sh           builds both in release mode, runs both benches, prints a table
```

## Rendering (identical in both)
- Use the real GPU, never `llvmpipe`: prefer DISCRETE, then INTEGRATED; skip CPU devices.
- One graphics pipeline, no vertex buffer, `vkCmdDraw(cmd, 6, instanceCount, 0, 0)`.
- Instance buffer: per-instance `{ vec2 pos; vec2 size; vec4 color }` = 32 bytes,
  binding 0, input rate INSTANCE, locations 0/1/2 (R32G32 / R32G32 / R32G32B32A32 SFLOAT).
  Board space is normalized [0,1]^2, origin top-left (see quad.vert).
- Host-visible + host-coherent instance buffer, persistently mapped, **one per frame in flight**.
- 2 frames in flight; per-frame fence + image-available semaphore; one render-finished
  semaphore **per swapchain image** (avoids the semaphore-reuse validation error).
- Clear colour `#14141c`. Alpha blending on. Swapchain format: prefer B8G8R8A8_SRGB, else first.
- Handle swapchain recreation on resize / OUT_OF_DATE / SUBOPTIMAL; handle a minimized window.
- Validation layers: enabled in debug builds only, never in release/bench.

## Game (identical rules)
- 20x20 grid, window 800x800, title "Vulkan Snake (C++)" / "Vulkan Snake (Rust)".
- Snake starts length 3 at the centre, moving right. Arrows / WASD steer.
- Input queue of at most 2 pending turns; reject 180° reversals against the last queued direction.
- Food on a random free cell. Eating: +1 score and growth by 1.
- Tick starts at 150 ms, minus 5 ms per food eaten, min 60 ms. Fixed-timestep logic, render every frame.
- Hitting a wall or itself = game over (snake turns red), R or Enter restarts, P/Space pauses, Esc quits.
- Window title is refreshed about twice a second: `... | Score N | Best N | FPS N`.
- Colours: head #7CFC00, body #32CD32 (darker toward the tail is fine), food #FF4757,
  board cells #1e1e2a with a 1-cell grid inset of 0.05 cell for a tile look, dead snake #c0392b.
- Pure game logic in its own module with no Vulkan or windowing code, plus unit tests:
  movement, growth, wall collision, self collision, reversal rejection, food never on the snake.
  C++: a `snake_tests` executable registered with CTest. Rust: `cargo test`.

## Benchmark mode — this is what gets compared
`--bench [--quads N] [--seconds S]` (defaults: N = 0, S = 10)
- Present mode: IMMEDIATE, else MAILBOX, else FIFO. Report which one was used.
- The snake is driven by an autopilot: each tick it steers greedily toward the food, never into
  a wall or its own body if a safe move exists, and the game restarts on death. RNG is seeded
  with a fixed value (seed 42, any simple PRNG such as xorshift32), so runs are repeatable.
- Every frame, also write N "stress" quads into the instance buffer from the CPU,
  with t = seconds since bench start:
  ```
  x = fract(i * 0.6180339 + t * 0.10)
  y = fract(i * 0.3819660 + t * 0.07 + 0.05 * sin(t + i * 0.001))
  size = 0.004 x 0.004
  color = (fract(i*0.13), fract(i*0.37), fract(i*0.71), 0.6)
  ```
  where `fract(v) = v - floor(v)`, computed in f32. This exercises the CPU side of each language.
- 2-second warm-up that is not measured, then S measured seconds.
- Record every frame time (CPU wall clock between successive presents).
- At the end print **exactly one line** to stdout and exit 0:
  ```
  {"impl":"cpp","present_mode":"IMMEDIATE","quads":N,"seconds":S,"frames":F,"avg_fps":X,"p1_low_fps":Y,"avg_ms":A,"p99_ms":B,"gpu":"<deviceName>"}
  ```
  `p1_low_fps` = 1000 / (mean of the slowest 1% of frame times in ms). `p99_ms` = 99th percentile frame time.
  Numbers rounded to 2 decimal places.

## Builds
- C++: `cmake -S vulkan/cpp -B vulkan/cpp/build -DCMAKE_BUILD_TYPE=Release && cmake --build vulkan/cpp/build`,
  binary at `vulkan/cpp/build/snake_vk`. Flags: `-O2`, no `-march=native` (Rust doesn't use it either).
- Rust: `cargo build --release` in vulkan/rust, binary at `vulkan/rust/target/release/snake_vk`.
  Use `[profile.release] opt-level = 3`, default codegen settings otherwise.
- Shaders are compiled at build time with glslc (C++: CMake custom command; Rust: build.rs
  that calls glslc), and the SPIR-V is embedded in or placed next to the binary.
