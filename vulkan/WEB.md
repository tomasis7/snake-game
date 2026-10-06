# Furious Snake on the web: Rust → WASM + WebGPU

The third port of the TS game, running in the browser. It reuses the Rust port's game code
unchanged and swaps only the platform layer: renderer, event loop, input, audio and assets.
It must look and play exactly like `vulkan/rust`. Everything in [SPEC.md](SPEC.md) about the game,
the canvas, the instance layout and the quad kinds still applies.

## Crate layout
```
vulkan/
  core/    furious_core: everything platform-independent, moved out of vulkan/rust/src
           (levels, sprites, player, robot, pathfinding, collision, camera, race, effects,
           progress, text, rng, input, button, screens, board, game, draw/instance building,
           atlas building from PNG bytes, bench stats). No ash, winit, rodio, web-sys or wgpu.
           Assets are passed in as bytes; core never touches the filesystem except through a
           trait or closure the platform provides (progress file, sound playback).
  rust/    the native app: Vulkan renderer, winit loop and rodio audio on top of furious_core.
  web/     the browser app: wgpu renderer, requestAnimationFrame loop and HTMLAudioElement
           audio on top of furious_core.
  shaders/quad.wgsl   a hand port of quad.vert + quad.frag (same kinds, same maths).
                      WebGPU has no push constants, so canvasSize is a 16-byte uniform buffer.
```
The native Rust behaviour must not change. `cargo test` in core and rust must still pass, with the
same 90 tests moved to wherever their code now lives. The native Rust release binary still builds
with no warnings, and its `--bench` and `--screenshot` still work.

## Web specifics
- **Build:** `vulkan/web/build.sh` runs:
  1. `cargo build --release --target wasm32-unknown-unknown`
  2. `wasm-bindgen --target web --out-dir vulkan/web/dist` (pin the `wasm-bindgen` crate with `=` to
     the installed CLI version)
  3. it then copies `index.html` and the TS game's mp3 files from `public/assets/` into `dist/`.

  Serve it with `python3 -m http.server 8642 -d vulkan/web/dist`. Add `vulkan/web/dist/` and
  `vulkan/web/target/` to the root `.gitignore`.
- **Assets:** levels, sprites.txt, font.png and background.png are embedded with `include_bytes!` and
  decoded with the `png` crate. The mp3s are fetched as files.
- **Renderer:**
  - `wgpu` with **WebGPU first, WebGL2 as the fallback** (features `webgpu`, `webgl`, `wgsl`).
    Try a WebGPU instance and adapter first. If `navigator.gpu` is missing or `requestAdapter()`
    returns null (the default in this machine's Chrome on Linux until
    `chrome://flags/#enable-unsafe-webgpu` is on), recreate everything on the WebGL2 backend.
    Use `wgpu::Limits::downlevel_webgl2_defaults()` there, and keep the shader, buffers and
    pipeline within what WebGL2 allows.
  - Report the backend in use: in the page title (`Furious Snake (WASM, WebGPU) | FPS N` or `(WASM, WebGL2)`) and in the bench JSON as `"api":"webgpu"` or `"api":"webgl2"`.
  - A `<canvas id="game">` drawn at 1200 × 800, CSS-scaled to fit the window at 3:2 and centred on black.
  - One pipeline and one instanced draw, with the same 64-byte instances that core builds.
  - The instance buffer is updated with `queue.write_buffer`.
  - The atlas uses a nearest sampler, with standard alpha blending and a non-sRGB (Unorm) surface format.
  - If `navigator.gpu` is missing or no adapter is found, show a readable message in the page instead of a blank canvas.
- **Loop:**
  - `requestAnimationFrame` drives frames.
  - The fixed 60 Hz simulation and render interpolation are the same as native, using core's code.
  - The game clock comes from `performance.now()`.
- **Input:**
  - Keyboard: `keydown` and `keyup` on `window`, mapped to the same keys as native. `preventDefault` stops the arrow keys and space from scrolling.
  - Mouse: positions are mapped into the 1200 × 800 canvas coordinates.
- **Audio:** each sound is an `HtmlAudioElement`, played at the same moments as the TS game.
  - Browsers block sound until the first click or keypress, so music and sounds start on that first gesture, like the TS game's `userStartAudio()`.
  - A `?mute` URL flag turns sound off.
- **Best times:** stored in `localStorage` under the TS game's key `furious-snake-best-time-L<n>`, so the web build shares best times with the TS game on the same origin.
- **Title:** the page title updates about twice a second to `Furious Snake (WASM) | FPS N`.

## Bench mode: `index.html?bench[&quads=N][&seconds=S][&level=L]`
This works the same way as the native bench: level-L race, both snakes on robot AI with mistake
chance 0, xorshift32 seed 42, the same stress-quad formula as SPEC.md, a 2 s warm-up, then S
measured seconds. The browser locks rAF to the display refresh rate, so the bench doesn't use rAF.
It runs its own loop:
- **Each frame:**
  - advance the simulation by the frame's real dt
  - build the instances
  - `write_buffer`, then render into an **offscreen 1200 × 800 texture**, with the same pipeline and the same format as the surface
  - submit
- **Two frames in flight:** before submitting frame N, wait until frame N−2's work is done (`queue.on_submitted_work_done` with a JS promise). Yield to the event loop at least once per frame.
- **Frame time:** the wall clock between successive frame starts, which includes the GPU through the in-flight limit.
- **Output:** the same JSON line as native, with `"impl":"wasm"` and `"present_mode":"offscreen"`. The
  `gpu` field comes from the adapter info and may be empty if the browser hides it. Print the line
  with `console.log` and also put it in `<pre id="bench-result">`.
- **Display:** while the bench runs, copy the offscreen texture to the canvas about once a second so it's visibly running.
