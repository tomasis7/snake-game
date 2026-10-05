# Vulkan Snake task list

- [x] Toolchain installed and checked (Vulkan 1.4, GLFW 3.4, glslc, CMake, Rust 1.98; GPU = Intel Iris Xe)
- [x] Shared spec and shaders (SPEC.md, shaders/)
- [x] C++ implementation (vulkan/cpp), built and tested
- [x] Rust implementation (vulkan/rust), built and tested
- [x] bench.sh plus FPS comparison runs (baseline and x86-64-v2)
- [x] Investigate Rust's 30% deficit on baseline x86-64 (libm floorf call; fixed by x86-64-v2)
- [x] Review, results in README, commit and push
