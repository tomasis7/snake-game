#pragma once
#include <cstdint>
#include <string>

#include "atlas.hpp"
#include "color.hpp"
#include "instance.hpp"
#include "textlayout.hpp"

// Writes quads straight into a mapped instance buffer, in draw order (painter's algorithm).
// All coordinates are canvas pixels (1200 x 800, origin top-left). Overflow is dropped.
class DrawList {
public:
    DrawList(Instance* buffer, uint32_t capacity, const Atlas& atlas)
        : buf_(buffer), cap_(capacity), atlas_(atlas) {}

    uint32_t count() const { return n_; }
    uint32_t capacity() const { return cap_; }
    // Claims up to n raw instances (for callers that fill them directly); returns the first and
    // sets n to the number actually granted.
    Instance* reserve(uint32_t& n) {
        if (n > cap_ - n_) n = cap_ - n_;
        Instance* p = buf_ + n_;
        n_ += n;
        return p;
    }

    void solid(float x, float y, float w, float h, Color c);
    void solidCentered(float cx, float cy, float w, float h, Color c) { solid(cx - w / 2, cy - h / 2, w, h, c); }
    void rounded(float x, float y, float w, float h, float radius, Color c);
    void roundedCentered(float cx, float cy, float w, float h, float radius, Color c) {
        rounded(cx - w / 2, cy - h / 2, w, h, radius, c);
    }
    // Outline (p5 stroke with no fill) of a centred rect: four solid bars, `weight` thick, centred on the edge.
    void strokeRectCentered(float cx, float cy, float w, float h, float weight, Color c);

    void sprite(SpriteId id, float cx, float cy, float w, float h, Color tint = {1, 1, 1, 1});
    void background(float x, float y, float w, float h);
    void text(const std::string& s, float x, float y, float size, HAlign align, Color c);
    // Text with a 1 px black outline on every side (p5 stroke("#000"), strokeWeight(2)).
    void textOutlined(const std::string& s, float x, float y, float size, HAlign align, Color c);

    void ball(float cx, float cy, float diameter, Color mid, Color edge, float centreMix);
    void shadow(float cx, float cy, float diameter, Color c);

private:
    Instance* next();

    Instance* buf_;
    uint32_t cap_;
    uint32_t n_ = 0;
    const Atlas& atlas_;
};
