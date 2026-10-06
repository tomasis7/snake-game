#!/usr/bin/env bash
# Builds the browser version into vulkan/web/dist (serve it with `python3 -m http.server 8642 -d vulkan/web/dist`).
set -euo pipefail
cd "$(dirname "$0")"
export PATH="$HOME/.cargo/bin:$PATH"

cargo build --release --target wasm32-unknown-unknown
rm -rf dist
mkdir -p dist
wasm-bindgen --target web --no-typescript --out-dir dist --out-name furious_web \
    target/wasm32-unknown-unknown/release/furious_web.wasm
cp index.html dist/
mkdir -p dist/assets/sounds dist/assets/music
cp ../../public/assets/sounds/*.mp3 dist/assets/sounds/
cp ../../public/assets/music/*.mp3 dist/assets/music/
ls -l dist
