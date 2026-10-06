# Furious Snake on Vulkan: C++ vs Rust

The TypeScript/p5.js game in `src/` ported twice to raw Vulkan, once in C++20 with GLFW and once in
Rust with `ash` and `winit`, both following [SPEC.md](SPEC.md). Both ports include:
- **Screens:** the menu, how to play, countdown, the two-snake race and results.
- **Gameplay:** the robot rival, all 3 levels, hearts, stars, plants and the ghost, the kill line, the zooming camera and the HUD.
- **Look:** the TS game's own pixel-art sprites, Press Start 2P font, parallax background and effects.

The two ports render identically: their screenshots match the TS game and each other.

Shared data is exported from the TS source, so there is one source of truth. To regenerate it:
`node vulkan/tools/export_data.mjs && python3 vulkan/tools/export_images.py`. The outputs are:
- `assets/levels/*.txt`
- `assets/sprites.txt`
- `assets/font.png`
- `assets/background.png`

## Run

Needs: `vulkan-headers vulkan-loader-devel vulkan-validation-layers glfw-devel glslc cmake rust cargo`.
Rust audio also needs `alsa-lib-devel` (on by default; use `--no-default-features` to build without sound).

```bash
# C++ (audio via miniaudio)
cmake -S vulkan/cpp -B vulkan/cpp/build -DCMAKE_BUILD_TYPE=Release && cmake --build vulkan/cpp/build -j
vulkan/cpp/build/snake_vk            # ctest --test-dir vulkan/cpp/build

# Rust
cd vulkan/rust && cargo run --release  # cargo test
```

Controls:
- **Race:** P1 uses the arrow keys and P2 uses WASD. G toggles the debug grid and Esc returns to the menu.
- **Menus:** click with the mouse, or use the keys: 1/2 pick the mode, Enter starts, H opens how to play, R retries and N goes to the next level.

Extra flags:
- `--mute` turns off sound.
- `--screenshot out.png --screen menu|howto|countdown|race|results` saves a screenshot.
- `--bench [--quads N] [--seconds S] [--level L]` runs the benchmark.

## Benchmark

```bash
vulkan/bench.sh [seconds] [runs]                # baseline x86-64
CPU=x86-64-v2 vulkan/bench.sh [seconds] [runs]  # same CPU target for both compilers
```

Bench mode plays the level-1 race with both snakes on the robot AI (seed 42), with vsync off.
It can also write N extra "stress" quads from the CPU each frame.

### Round 2, the full Furious Snake scene: Intel Iris Xe, 10 s x 2 runs

| quads | baseline C++ | baseline Rust | x86-64-v2 C++ | x86-64-v2 Rust |
|------:|----:|----:|----:|----:|
| 0 | ~2,600–3,200* | ~2,600–3,000* | 2,537 | 2,358 |
| 10,000 | 606 | 486 (−20%) | 807 | 957 (+19%) |
| 100,000 | 129 | 90 (−30%) | 180 | 160 (−11%) |

\* The 0-quad baseline row in the full run was ruined by a system hitch (554 and 559 FPS with 25 FPS
lows). These are standalone 5-second reruns instead.

Run-to-run noise on this laptop is about ±10%, so the gaps at 0 and 100k quads on v2 are within noise.
- **Baseline x86-64, Rust is 20–30% behind under CPU load.** Rust/LLVM compiles `f32::floor` as a
  call to libm's `floorf` when SSE4.1 isn't available, while GCC inlines it. A plain-RAM
  microbenchmark of the stress loop alone shows the same 6.4 ms vs 9.3 ms gap.
- **With x86-64-v2 for both,** the codegen gap disappears. The two are roughly even, and Rust leads in the 10k case.
- **Without stress quads,** both reach 2,000–3,000 FPS with a full game scene: sprites, text,
  gradient snakes, particles and the robot AI. The language barely matters for the Vulkan side.

(The stress pattern's y multiplier changed after these runs, from 0.382 to 0.755. The old value
lined every quad up on one diagonal stripe; the GPU cost is the same.)

### vs the original TypeScript/p5.js game in Chrome
I measured once in Chrome 156 on the same laptop. As in the Vulkan bench, both snakes were on the
robot AI in the level-1 race, after a 2 s warm-up. The browser caps p5 at the display's refresh rate,
so I stopped the loop with `noLoop()` and called `redraw()` back to back for 10 s, with a fixed
16.67 ms `deltaTime`:

| | avg frame | FPS from frame cost | 1% low |
|---|---:|---:|---:|
| TS p5.js (Canvas 2D) | 5.59 ms | **~180** (actual loop: 112) | 32 |
| C++ / Rust (Vulkan) | 0.3–0.4 ms | **~2,400–3,200** | ~260–350 |

On the same scene, the Vulkan ports do each frame about 15x faster. Every sprite, ball and glyph is
a separate Canvas 2D call in p5, while Vulkan sends one instanced draw per frame. In normal play the TS
game runs at `frameRate(60)` with plenty of headroom, so all three feel the same on a 60 Hz screen.
The browser number is main-thread time only: Chrome rasterises the canvas in its GPU process, so the
real cost is a bit higher.

### Rust → WASM + WebGPU in Chrome (`vulkan/web`, see [WEB.md](WEB.md))
The same Rust game code, shared through `vulkan/core`, compiled to WebAssembly and drawn with
`wgpu`. It uses WebGPU and falls back to WebGL2 when WebGPU is unavailable. Build it with
`vulkan/web/build.sh`, serve it with `python3 -m http.server 8642 -d vulkan/web/dist` and open
`http://127.0.0.1:8642/`. Add `?bench&seconds=10` to the URL for the benchmark.

One 10 s bench run in Chrome 156 on WebGPU, on the same robot-vs-robot level-1 race. It renders
offscreen with 2 frames in flight, so it isn't capped by the display:

| Version | avg frame | avg FPS | 1% low |
|---|---:|---:|---:|
| TS p5.js (Canvas 2D) | 5.59 ms | ~180 | 32 |
| **Rust → WASM + WebGPU** | **0.92 ms** | **1,081** | **505** |
| C++ / Rust native Vulkan | 0.3–0.4 ms | ~2,400–3,200 | ~260–350 |

In the browser, WASM + WebGPU is about 6x faster than p5.js, with much steadier frames: its 1% low
beats both native versions. Native Vulkan is still about 2.5–3x faster on average, because it has
no browser compositor, JS/WASM boundary or WebGPU validation layer in the way.

The WebGL2 fallback is built but untested here, because this Chrome has WebGPU enabled.

### Round 1, classic 20x20 snake (earlier commit)
Baseline: Rust −3% / −24% / −31% at 0 / 10k / 100k quads. x86-64-v2: −3% / +14% / +19%.
