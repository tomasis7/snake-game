# Furious Snake on Vulkan: shared spec (C++ and Rust)

Both ports recreate the **TypeScript/p5.js game in `src/`** ("Furious Snake", a two-snake
side-scrolling race) as faithfully as possible, written directly against Vulkan. Both
follow this spec, so they look and play the same and their frame rates can still be
compared fairly.

**The TypeScript source is the reference.** When this spec says "port X", read the named
`src/*.ts` file and reproduce its rules, constants, timings, colours, text and layout exactly.
p5 drawing calls map onto the quad kinds below.

## Layout
```
vulkan/
  SPEC.md
  shaders/quad.vert, quad.frag   shared GLSL (do not edit; compile with glslc at build time)
  assets/                        shared, generated from the TS game by vulkan/tools/
    levels/level{1,2,3}.txt      one line per row, one digit per 32 px tile
    sprites.txt                  pixel-art sprites (format below)
    font.png                     Press Start 2P, ASCII 32..127, 16 x 6 grid of 8 x 8 cells
    background.png               1472 x 832, the TS bakgrund.gif
  tools/                         export_data.mjs, export_images.py (already run; don't rerun)
  cpp/                           C++20 + Vulkan + GLFW, CMake
  rust/                          Rust + ash + winit, cargo
  bench.sh
```
Audio is loaded straight from the TS game's `public/assets/sounds/*.mp3` and
`public/assets/music/background-theme.mp3`. Resolve asset directories at compile time:
C++ uses CMake defines pointing at `vulkan/assets` and `public/assets`; Rust uses
`concat!(env!("CARGO_MANIFEST_DIR"), "/../assets")` and `/../../public/assets`.

`sprites.txt` format, repeated per sprite:
```
sprite <name> <cols> <rows>
<char> #rrggbb          one line per palette entry
rows
<row text>              <rows> lines, '.' = transparent
end
```
The sprites are star, starBright, heart, ghost, plant, wall, tetris and win.

## Renderer (identical in both)
- **Canvas:** a 1200 x 800 virtual canvas, the TS `createCanvas(1200, 800)`.
  - The window opens at 1200 x 800 and is resizable.
  - The canvas is letterboxed into it at 3:2 by setting the viewport and scissor to the
    largest centred 3:2 rect, and the area outside is cleared black.
  - Mouse coordinates are mapped back into canvas pixels.
- **GPU selection:** use the real GPU, never `llvmpipe`. Prefer DISCRETE, then INTEGRATED, and skip CPU devices.
- **Swapchain format:** prefer **B8G8R8A8_UNORM**, otherwise the first available. Colours are
  plain sRGB values with no linearising, so alpha blending matches the browser canvas.
  Standard alpha blending.
- **Pipeline and buffers:**
  - One pipeline, no vertex buffer, `vkCmdDraw(6, instanceCount)`.
  - Push constant `vec2 canvasSize = (1200, 800)` in the vertex stage.
  - The instance buffer is host-visible and host-coherent, persistently mapped, one per frame in flight.
  - Draw order is the order instances are written (painter's algorithm, no depth buffer).
- **Instance layout (64 bytes,** binding 0, INSTANCE rate, all R32G32[B32A32]_SFLOAT):
  `vec2 pos (loc 0), vec2 size (loc 1), vec4 color (loc 2), vec4 uv (loc 3), vec4 params (loc 4)`.
  `pos` is the top-left corner in canvas pixels. The kinds are in `params.x`, documented in quad.frag:
  - 0 SOLID: rects, HUD, buttons, particles, flash overlay.
  - 1 TEXTURED: sprites, text and background.
  - 2 BALL: snake segments.
  - 3 SHADOW: soft round shadow.
  - 4 ROUNDED: the progress bar and marker pills.
- **Atlas:** one RGBA8_UNORM texture built on the CPU at startup and uploaded once with a
  staging buffer. The layout is your choice, for example 2048 x 2048. It holds:
  - background.png
  - every sprite rasterised at 1 texel per sprite pixel, with 1 transparent texel of padding
  - font.png
  - one opaque white texel for untextured use
  It uses a NEAREST sampler with CLAMP_TO_EDGE and is bound as one combined image sampler at set 0, binding 0.
- **Text:** the p5 `text()` equivalent draws the font atlas as textured quads.
  - Each glyph is a square of `textSize` canvas pixels, with an advance of `textSize`.
    Press Start 2P is monospaced with 8 px cells.
  - Support LEFT, CENTER and RIGHT horizontal alignment and CENTER vertical alignment, like the TS calls.
- **Camera:** world-to-canvas transforms happen on the CPU, as in GameBoard.draw:
  translate, scale, translate, plus the shake offset. Cull off-screen entities with
  `visibleWorldRect` and `intersectsRect`, like the TS.
- **Frames in flight and sync:**
  - 2 frames in flight, a fence and an image-available semaphore per frame, and a render-finished semaphore per swapchain image.
  - Recreate the swapchain on resize, OUT_OF_DATE or SUBOPTIMAL, and handle a minimised window.
- **Validation:** validation layers are on in debug builds only, and there must be no validation errors.
- **Present mode:** the interactive game uses FIFO (vsync).
- **Window title:** `Furious Snake (C++) | FPS N` or `Furious Snake (Rust) | FPS N`, refreshed about twice a second.

## Game: port these TS files
- **Screens**, ported from game.ts, startmenu.ts, interactionscreen.ts, countdown.ts,
  gameboard.ts, resultsscreen.ts and button.ts:
  - Menu, How to play, Countdown, GameBoard (race) and Results, with the same text, positions, colours and sizes.
  - Buttons are clicked with the mouse, keeping button.ts's "press already held" guard.
  - Convenience keys, an addition to the TS:
    - Menu: 1 and 2 select the mode, Enter starts, H opens How to play.
    - How to play: Esc or Enter goes back.
    - Results: N next level, R retry, M or Esc goes to the menu.
    - In a race, Esc returns to the menu. On the menu, Esc quits.
- **Gameplay:**
  - **Player** (player.ts): 32 px grid steps on a 300 ms cycle (the move timer resets to −100 at ≥200), 8 starting segments, smooth interpolation between steps, polled key input, eyes on the head, hit blink, stun, 3 lives (max 10).
  - **P1** uses the arrow keys. **P2** uses WASD in 2-player mode, or **RobotPlayer** in 1-player mode (robotplayer.ts and ai/pathfinding.ts, with the per-level `robotMistakeChance` 0.25, 0.1 and 0).
  - **Collisions** (collisionmanager.ts): hazards are Block, TetrisBlock, Plant and Ghost; plus Star, Heart and the finish WinBlock. Keep the exact rules: the cooldown, the shake(10), flash and burst calls, and treating `position` as the top-left for collision.
  - **Entities** (block, tetrisBlocks, winBlock, star, heart, plant, ghost .ts): each sprite is drawn centred on `position` at `size`, with the heart pulse, star twinkle and ghost bob and drift.
  - **Levels** (levelfactory.ts): tile at `(col*32 + 16, row*32 + 16)`, with codes 1–7 as in the TS.
  - **Race** (camera.ts, racemanager.ts): `fitCamera` with padding 220 and scale 0.5–1.0, a kill line with MAX_GAP 1000, the flashing "OUT OF TIME!" warning, the HUD (level label, progress bar with markers and finish flag, clock, lives pips) and `formatTime`.
  - **Background**: drawn exactly as in GameBoard.draw, as 1415 x 800 tiles with `cam.centerX * 0.25` parallax.
  - **Effects** (effects/effects.ts): particles, shake, flash and floating text with the same constants. Use `Math.random()` equivalents from your own PRNG.
  - **Progress** (progress.ts): best time per level is saved to
    `${XDG_DATA_HOME:-~/.local/share}/furious-snake-vulkan/best_times.txt`, one line per level, `L<n> <ms>`. Both ports share this file. Missing or unreadable files are ignored.
  - **Debug grid**: G toggles it (main.ts).
- **Snake look:** match player.ts.
  - Each segment is a SHADOW quad (rgba(0,0,0,0.3), offset +5,+5, about 1.6x the diameter, for the blur 15) followed by a BALL quad.
  - Head: mid colour #FFA500, edge #804600, centre mix 0.8, which approximates #FFE5CC.
  - Body: mid = `trailFillColor`, edge = lerp(`trailStrokeColor`, black, 0.7), centre mix 0.2.
  - P1 is #00FFFF with stroke "green", P2 is #FF00FF with stroke "orange".
  - Eyes are SOLID rects.
- **Timing:**
  - Simulation runs at a fixed 60 Hz, with dt = 16.667 ms as the p5 `deltaTime`. Per-frame values such as the ghost's 0.3 px/frame match p5 at `frameRate(60)`.
  - Rendering runs every frame and interpolates the snakes with the leftover accumulator time: `moveProgress = clamp((moveTimer + acc + 100) / 300, 0, 1)`.
  - TS code that uses `Date.now()` or `millis()` uses the game clock.
- **Audio:** play the same sounds at the same moments as the TS (`sounds.*`), and loop the background music from the start.
  - C++ uses **miniaudio** (fetched with CMake FetchContent). Rust uses **rodio** with mp3 support.
  - If no audio device opens, print one warning and continue silently.
  - `--mute` disables audio. Bench and screenshot runs are always muted.
- **Pure logic:** keep it in modules with no Vulkan, window or audio code, so it can be unit-tested:
  levels, sprite parsing, player, robot and pathfinding, collisions, camera, race, effects and progress.

## Tests
Port the TS tests as unit tests: camera.test.ts, racemanager.test.ts, progress.test.ts,
ai/pathfinding.test.ts, effects/effects.test.ts and art/sprites.test.ts. Also add:
- level loading: tile counts per type for each level, and the WinBlock column
- player stepping: position after one 300 ms cycle, no reversal into itself
- collision rules: a hazard costs 1 life, starts the cooldown and stuns; a heart is capped at 10 lives; touching the WinBlock finishes the race

C++ uses a `snake_tests` CTest executable. Rust uses `cargo test`.

## Benchmark mode: what gets compared
`--bench [--quads N] [--seconds S] [--level L]` (defaults N = 0, S = 10, L = 1)
- **Present mode:** IMMEDIATE, else MAILBOX, else FIFO.
- **Setup:** skips the menu and countdown and runs the level-L race with **both** snakes driven by the robot AI, with mistake chance 0.
  - The PRNG is xorshift32 with seed 42 for everything: AI, effects and particles.
  - The race restarts on the same level when it ends.
- **Stress quads:** each frame, after the scene, also write N SOLID quads, with t = seconds since the loop started (including warm-up):
  ```
  x = fract(i * 0.6180339 + t * 0.10) * 1200
  y = fract(i * 0.7548777 + t * 0.07 + 0.05 * sin(t + i * 0.001)) * 800
  size = 4.8 x 4.8
  color = (fract(i*0.13), fract(i*0.37), fract(i*0.71), 0.6)
  ```
  `fract(v) = v - floor(v)`, all in f32.
- **Measurement:**
  - 2 s of warm-up are not measured, then S measured seconds.
  - Frame time is the wall-clock gap between successive loop iterations.
  - `avg_fps` = frames / sum of frame times, `p99_ms` is nearest-rank, and `p1_low_fps` = 1000 / mean of the slowest max(1, n/100) frames.
- **Output:** print exactly one line to stdout and exit 0:
  `{"impl":"cpp","present_mode":"IMMEDIATE","quads":N,"seconds":S,"frames":F,"avg_fps":X,"p1_low_fps":Y,"avg_ms":A,"p99_ms":B,"gpu":"<deviceName>"}`
  Numbers are rounded to 2 decimal places.

## Screenshot mode (for checking the look)
`--screenshot <out.png> --screen menu|howto|countdown|race|results [--after-ms T]` (default T = 1500)
- **Setup:** opens the given screen directly.
  - `race` behaves like bench mode: both snakes are on robot AI with seed 42.
  - `results` shows a level-1 1-player win at time 42.3 s with "NEW BEST!".
- **Run:** FIFO present mode. The game runs for T ms of game time, then the canvas area of the next frame is captured.
  - The swapchain needs TRANSFER_SRC usage.
  - Copy the image to a host buffer, crop it to the letterboxed canvas, convert BGRA to RGBA and write a PNG.
  - Exit 0 afterwards.
  - PNG writing: C++ uses stb_image_write (stb via FetchContent); Rust uses the `png` crate.
  - Loading PNGs: C++ uses stb_image; Rust uses the `png` crate.

## Builds
- **C++:** `cmake -S vulkan/cpp -B vulkan/cpp/build -DCMAKE_BUILD_TYPE=Release && cmake --build vulkan/cpp/build -j`,
  binary at `vulkan/cpp/build/snake_vk`. Flags: `-O2`, no `-march` unless bench.sh's `CPU` sets it.
- **Rust:** `cargo build --release`, binary at `vulkan/rust/target/release/snake_vk`, with `opt-level = 3`.
- **Libraries:** no third-party libraries beyond those named here.
  - C++: Vulkan, GLFW, miniaudio and stb.
  - Rust: ash, ash-window, winit, raw-window-handle, png and rodio.
