#pragma once
#include <array>
#include <cstdint>
#include <string>
#include <vector>

struct UvRect {
    float u0 = 0, v0 = 0, u1 = 0, v1 = 0;
};

enum SpriteId { SPR_STAR, SPR_STAR_BRIGHT, SPR_HEART, SPR_GHOST, SPR_PLANT, SPR_WALL, SPR_TETRIS, SPR_WIN, SPR_COUNT };

// CPU-built texture atlas (RGBA8) plus the UV lookups into it. No Vulkan here.
struct Atlas {
    int width = 0, height = 0;
    std::vector<uint8_t> rgba;
    UvRect background;                   // the whole 1472 x 832 image
    std::array<UvRect, SPR_COUNT> sprites;
    float fontU0 = 0, fontV0 = 0;        // origin of the 128 x 48 font grid
    float texelU = 0, texelV = 0;        // one texel in uv units
    UvRect white;                        // an opaque white texel

    UvRect glyph(int index) const;       // 0..95
};

// Loads background.png, font.png and sprites.txt from `assetDir`. Throws std::runtime_error on failure.
Atlas buildAtlas(const std::string& assetDir);
