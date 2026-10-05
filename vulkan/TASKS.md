# Vulkan Snake task list

## Round 1: classic snake and FPS comparison (done)
- [x] Toolchain, shared spec and shaders, C++ and Rust ports, bench.sh, results in README

## Round 2: port the TS "Furious Snake" game (done)
- [x] Survey TS game, export shared assets (vulkan/tools/), new shaders, SPEC.md v2
- [x] C++ port (66 tests), Rust port (90 tests), screenshots match the TS game
- [x] Benchmarks on the real race scene, README updated
- [x] Fix the stress-quad pattern (it all landed on one diagonal line)

## Open
- [x] Rust audio: alsa-lib-devel installed, `audio` is now a default feature, builds and runs
- [ ] Optional: a cleaner benchmark run (3+ runs, nothing else running)
