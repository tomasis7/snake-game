# Vulkan Snake: C++ vs Rust

The same Snake game written twice against raw Vulkan, once in C++20 with GLFW and once in Rust
with `ash` and `winit`, so their frame rates can be compared fairly. Both follow
[SPEC.md](SPEC.md): the same shaders, one instanced draw call, 2 frames in flight, the same game
rules and the same benchmark mode.

## Run

Needs: `vulkan-headers vulkan-loader-devel vulkan-validation-layers glfw-devel glslc cmake rust cargo`.

```bash
# C++
cmake -S vulkan/cpp -B vulkan/cpp/build -DCMAKE_BUILD_TYPE=Release && cmake --build vulkan/cpp/build -j
vulkan/cpp/build/snake_vk
ctest --test-dir vulkan/cpp/build

# Rust
cd vulkan/rust && cargo run --release
cargo test
```

Controls: arrow keys or WASD to steer, P or Space to pause, R or Enter to restart, Esc to quit.

## Benchmark

```bash
vulkan/bench.sh [seconds] [runs]                # baseline x86-64
CPU=x86-64-v2 vulkan/bench.sh [seconds] [runs]  # same CPU target for both compilers
```

Bench mode (`--bench --quads N --seconds S`) disables vsync with IMMEDIATE present mode and lets
an autopilot play. Every frame it also writes N animated "stress" quads from the CPU into the
mapped instance buffer. Runs alternate between the two binaries.

### Results: Intel Iris Xe (TGL GT2), Mesa, Wayland, 10 s × 2 runs

Baseline x86-64 (default compiler settings):

| quads | C++ FPS | Rust FPS | Rust vs C++ |
|------:|--------:|---------:|------------:|
| 0 | 3950 | 3827 | −3% |
| 10,000 | 897 | 684 | −24% |
| 100,000 | 147 | 101 | −31% |

`CPU=x86-64-v2` (both compilers):

| quads | C++ FPS | Rust FPS | Rust vs C++ |
|------:|--------:|---------:|------------:|
| 0 | 3868 | 3764 | −3% |
| 10,000 | 1205 | 1377 | **+14%** |
| 100,000 | 210 | 250 | **+19%** |

### What explains the numbers

- **Empty scene (0 quads):** both run at about 3,800–3,950 FPS, within noise of each other. Here the
  frame cost is mostly the driver and the compositor, and the language makes no difference.
- **Baseline x86-64, Rust is about 30% slower.** The stress loop calls `floor` five times per quad.
  Without SSE4.1, Rust/LLVM compiles `f32::floor` as a **call to libm's `floorf`**, while GCC
  inlines `std::floor` with a short `cvttss2si`/`cvtsi2ss` sequence. A plain-RAM microbenchmark of the
  loop alone gives 6.4 ms vs 9.3 ms per 100k quads, which is exactly the in-game gap. So the renderers
  are equal and the difference is codegen for a single function.
- **x86-64-v2 (SSE4.1), Rust is 14–19% faster.** Both compilers now emit `roundss` for floor, and
  LLVM vectorises the rest of the loop better than GCC (microbenchmark: 3.2 ms vs 3.8 ms). Both
  versions also get 35–150% faster just from the CPU target.

In short, the language barely matters for Vulkan itself, because the GPU and driver do the heavy
lifting. CPU-side codegen details, such as the target CPU level and how `floor` is compiled,
matter far more than C++ vs Rust.
