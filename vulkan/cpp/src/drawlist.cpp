#include "drawlist.hpp"

#include <vector>

Instance* DrawList::next() { return n_ < cap_ ? &buf_[n_++] : nullptr; }

static void fill(Instance* q, float x, float y, float w, float h, Color c, float kind, float p1 = 0,
                 float p2 = 0) {
    q->pos[0] = x;
    q->pos[1] = y;
    q->size[0] = w;
    q->size[1] = h;
    q->color[0] = c.r;
    q->color[1] = c.g;
    q->color[2] = c.b;
    q->color[3] = c.a;
    q->uv[0] = q->uv[1] = q->uv[2] = q->uv[3] = 0;
    q->params[0] = kind;
    q->params[1] = p1;
    q->params[2] = p2;
    q->params[3] = 0;
}

void DrawList::solid(float x, float y, float w, float h, Color c) {
    if (Instance* q = next()) fill(q, x, y, w, h, c, KIND_SOLID);
}

void DrawList::rounded(float x, float y, float w, float h, float radius, Color c) {
    if (Instance* q = next()) fill(q, x, y, w, h, c, KIND_ROUNDED, radius);
}

void DrawList::strokeRectCentered(float cx, float cy, float w, float h, float weight, Color c) {
    float l = cx - w / 2 - weight / 2, r = cx + w / 2 + weight / 2;
    float t = cy - h / 2 - weight / 2, b = cy + h / 2 + weight / 2;
    solid(l, t, r - l, weight, c);
    solid(l, b - weight, r - l, weight, c);
    solid(l, t + weight, weight, b - t - 2 * weight, c);
    solid(r - weight, t + weight, weight, b - t - 2 * weight, c);
}

static void setUv(Instance* q, const UvRect& uv) {
    q->uv[0] = uv.u0;
    q->uv[1] = uv.v0;
    q->uv[2] = uv.u1;
    q->uv[3] = uv.v1;
}

void DrawList::sprite(SpriteId id, float cx, float cy, float w, float h, Color tint) {
    if (Instance* q = next()) {
        fill(q, cx - w / 2, cy - h / 2, w, h, tint, KIND_TEXTURED);
        setUv(q, atlas_.sprites[id]);
    }
}

void DrawList::background(float x, float y, float w, float h) {
    if (Instance* q = next()) {
        fill(q, x, y, w, h, {1, 1, 1, 1}, KIND_TEXTURED);
        setUv(q, atlas_.background);
    }
}

void DrawList::text(const std::string& s, float x, float y, float size, HAlign align, Color c) {
    static thread_local std::vector<Glyph> glyphs;
    glyphs.clear();
    layoutText(s, x, y, size, align, glyphs);
    for (const Glyph& g : glyphs) {
        Instance* q = next();
        if (!q) return;
        fill(q, g.x, g.y, g.size, g.size, c, KIND_TEXTURED);
        setUv(q, atlas_.glyph(g.index));
    }
}

void DrawList::textOutlined(const std::string& s, float x, float y, float size, HAlign align, Color c) {
    for (int dy = -1; dy <= 1; ++dy)
        for (int dx = -1; dx <= 1; ++dx)
            if (dx || dy) text(s, x + dx, y + dy, size, align, {0, 0, 0, c.a});
    text(s, x, y, size, align, c);
}

void DrawList::ball(float cx, float cy, float d, Color mid, Color edge, float centreMix) {
    if (Instance* q = next()) {
        fill(q, cx - d / 2, cy - d / 2, d, d, mid, KIND_BALL, centreMix, d);
        q->uv[0] = edge.r;
        q->uv[1] = edge.g;
        q->uv[2] = edge.b;
        q->uv[3] = edge.a;
    }
}

void DrawList::shadow(float cx, float cy, float d, Color c) {
    if (Instance* q = next()) fill(q, cx - d / 2, cy - d / 2, d, d, c, KIND_SHADOW);
}
