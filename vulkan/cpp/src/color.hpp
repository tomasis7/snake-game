#pragma once
#include <string>

struct Color {
    float r = 0, g = 0, b = 0, a = 1;
};

// "#rrggbb" or one of: white, black, green, orange (CSS values, as p5 resolves them).
Color hex(const std::string& s);
Color withAlpha(Color c, float a);
Color lerpColor(Color a, Color b, float t);
