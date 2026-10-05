#include "atlas.hpp"

#include <stb_image.h>

#include <cstdlib>
#include <cstring>
#include <stdexcept>

#include "sprites.hpp"

namespace {

constexpr int ATLAS_W = 2048;
constexpr int ATLAS_H = 1024;

struct Image {
    int w = 0, h = 0;
    uint8_t* data = nullptr;
    ~Image() { stbi_image_free(data); }
};

void loadPng(const std::string& path, Image& img) {
    int comp = 0;
    img.data = stbi_load(path.c_str(), &img.w, &img.h, &comp, 4);
    if (!img.data) throw std::runtime_error("cannot load " + path);
}

void blit(std::vector<uint8_t>& dst, int dx, int dy, const uint8_t* src, int w, int h) {
    for (int y = 0; y < h; ++y)
        std::memcpy(&dst[((size_t)(dy + y) * ATLAS_W + dx) * 4], src + (size_t)y * w * 4, (size_t)w * 4);
}

uint32_t parseHex(const std::string& s) { return (uint32_t)std::strtoul(s.c_str() + 1, nullptr, 16); }

}  // namespace

UvRect Atlas::glyph(int index) const {
    int cx = (index % 16) * 8, cy = (index / 16) * 8;
    // Inset a quarter texel so nearest sampling never bleeds in a neighbouring cell.
    float e = 0.25f;
    return {fontU0 + (cx + e) * texelU, fontV0 + (cy + e) * texelV, fontU0 + (cx + 8 - e) * texelU,
            fontV0 + (cy + 8 - e) * texelV};
}

Atlas buildAtlas(const std::string& assetDir) {
    Atlas a;
    a.width = ATLAS_W;
    a.height = ATLAS_H;
    a.rgba.assign((size_t)ATLAS_W * ATLAS_H * 4, 0);
    a.texelU = 1.0f / ATLAS_W;
    a.texelV = 1.0f / ATLAS_H;
    auto uv = [&](float x0, float y0, float x1, float y1, float inset) {
        return UvRect{(x0 + inset) * a.texelU, (y0 + inset) * a.texelV, (x1 - inset) * a.texelU,
                      (y1 - inset) * a.texelV};
    };

    Image bg;
    loadPng(assetDir + "/background.png", bg);
    blit(a.rgba, 0, 0, bg.data, bg.w, bg.h);
    a.background = uv(0, 0, (float)bg.w, (float)bg.h, 0.25f);

    Image font;
    loadPng(assetDir + "/font.png", font);
    int fontY = bg.h + 4;
    blit(a.rgba, 0, fontY, font.data, font.w, font.h);
    a.fontU0 = 0;
    a.fontV0 = fontY * a.texelV;

    std::vector<Sprite> sprites;
    std::string err;
    if (!loadSprites(assetDir + "/sprites.txt", sprites, err)) throw std::runtime_error(err);
    static const char* names[SPR_COUNT] = {"star", "starBright", "heart", "ghost", "plant", "wall", "tetris", "win"};
    int cursorX = 2;
    int spriteY = fontY + font.h + 4;
    for (int id = 0; id < SPR_COUNT; ++id) {
        const Sprite* s = findSprite(sprites, names[id]);
        if (!s) throw std::runtime_error(std::string("missing sprite ") + names[id]);
        for (int r = 0; r < s->rowsCount; ++r) {
            for (int c = 0; c < s->cols; ++c) {
                char ch = s->rows[r][c];
                if (ch == '.') continue;
                auto it = s->palette.find(ch);
                if (it == s->palette.end()) throw std::runtime_error("sprite colour missing");
                uint32_t rgb = parseHex(it->second);
                uint8_t* p = &a.rgba[((size_t)(spriteY + r) * ATLAS_W + cursorX + c) * 4];
                p[0] = (rgb >> 16) & 255;
                p[1] = (rgb >> 8) & 255;
                p[2] = rgb & 255;
                p[3] = 255;
            }
        }
        a.sprites[id] = uv((float)cursorX, (float)spriteY, (float)(cursorX + s->cols), (float)(spriteY + s->rowsCount), 0.0f);
        cursorX += s->cols + 2;  // 1 transparent texel of padding each side
    }

    // One opaque white texel in the bottom-right corner.
    uint8_t* w = &a.rgba[((size_t)(ATLAS_H - 1) * ATLAS_W + (ATLAS_W - 1)) * 4];
    w[0] = w[1] = w[2] = w[3] = 255;
    a.white = uv(ATLAS_W - 1, ATLAS_H - 1, ATLAS_W, ATLAS_H, 0.25f);
    return a;
}
