#include "textlayout.hpp"

#include <algorithm>

namespace {
std::vector<std::string> splitLines(const std::string& t) {
    std::vector<std::string> lines(1);
    for (char c : t) {
        if (c == '\n') lines.emplace_back();
        else lines.back().push_back(c);
    }
    return lines;
}
}  // namespace

float textWidth(const std::string& text, float size) {
    float w = 0;
    for (auto& l : splitLines(text)) w = std::max(w, l.size() * size);
    return w;
}

void layoutText(const std::string& text, float x, float y, float size, HAlign align,
                std::vector<Glyph>& out) {
    auto lines = splitLines(text);
    float leading = size * 1.25f;
    float firstCentre = y - (lines.size() - 1) * leading / 2;
    for (size_t li = 0; li < lines.size(); ++li) {
        const std::string& line = lines[li];
        float width = line.size() * size;
        float left = x;
        if (align == HAlign::Center) left = x - width / 2;
        else if (align == HAlign::Right) left = x - width;
        float top = firstCentre + li * leading - size / 2;
        for (size_t i = 0; i < line.size(); ++i) {
            unsigned char c = (unsigned char)line[i];
            if (c == ' ') continue;
            if (c < 32 || c > 127) c = '?';
            out.push_back({left + i * size, top, size, c - 32});
        }
    }
}
