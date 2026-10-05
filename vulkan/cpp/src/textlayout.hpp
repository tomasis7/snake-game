#pragma once
#include <string>
#include <vector>

enum class HAlign { Left, Center, Right };

struct Glyph {
    float x, y, size;  // top-left of the square glyph cell, canvas pixels
    int index;         // 0..95, ASCII 32..127 in the 16 x 6 font atlas
};

// p5 text() with the given horizontal alignment and CENTER vertical alignment. Each glyph is a
// textSize square with an advance of textSize; '\n' starts a new line (leading 1.25 * size).
// Spaces advance but emit no glyph. Non-ASCII bytes become '?'.
void layoutText(const std::string& text, float x, float y, float size, HAlign align,
                std::vector<Glyph>& out);
float textWidth(const std::string& text, float size);  // widest line
