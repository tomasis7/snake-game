"""Exports the TypeScript game's font and background into PNGs for the Vulkan ports.

Run from the repo root: python3 vulkan/tools/export_images.py

font.png: Press Start 2P (the TS game's font) rasterised at its native 8 px
grid, ASCII 32..127 in a 16 x 6 grid of 8 x 8 cells, white on transparent.
background.png: the first frame of bakgrund.gif.
"""
from PIL import Image, ImageDraw, ImageFont

OUT = "vulkan/assets"
CELL = 8

font = ImageFont.truetype("public/assets/fonts/PressStart2P-Regular.ttf", CELL)
atlas = Image.new("LA", (16 * CELL, 6 * CELL), (255, 0))
draw = ImageDraw.Draw(atlas)
draw.fontmode = "1"  # no antialiasing: the font is pixel art
for code in range(32, 128):
    i = code - 32
    x, y = (i % 16) * CELL, (i // 16) * CELL
    draw.text((x, y), chr(code), font=font, fill=(255, 255))
atlas.convert("RGBA").save(f"{OUT}/font.png")

bg = Image.open("public/assets/images/bakgrund.gif").convert("RGBA")
bg.save(f"{OUT}/background.png", optimize=True)
print("font.png", atlas.size, "background.png", bg.size)
