#include "color.hpp"

#include <cstdlib>

Color hex(const std::string& s) {
    if (s == "white") return {1, 1, 1, 1};
    if (s == "black") return {0, 0, 0, 1};
    if (s == "green") return {0.0f, 128.0f / 255.0f, 0.0f, 1};
    if (s == "orange") return {1.0f, 165.0f / 255.0f, 0.0f, 1};
    if (s.size() == 7 && s[0] == '#') {
        unsigned v = (unsigned)std::strtoul(s.c_str() + 1, nullptr, 16);
        return {((v >> 16) & 255) / 255.0f, ((v >> 8) & 255) / 255.0f, (v & 255) / 255.0f, 1};
    }
    return {};
}

Color withAlpha(Color c, float a) {
    c.a = a;
    return c;
}

Color lerpColor(Color a, Color b, float t) {
    return {a.r + (b.r - a.r) * t, a.g + (b.g - a.g) * t, a.b + (b.b - a.b) * t,
            a.a + (b.a - a.a) * t};
}
