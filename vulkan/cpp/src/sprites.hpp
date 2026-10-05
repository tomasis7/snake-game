#pragma once
#include <map>
#include <string>
#include <vector>

struct Sprite {
    std::string name;
    int cols = 0, rowsCount = 0;
    std::vector<std::string> rows;        // '.' = transparent
    std::map<char, std::string> palette;  // char -> "#rrggbb"
};

// Parses the vulkan/assets/sprites.txt format. Returns false (and sets err) on malformed input.
bool parseSprites(const std::string& text, std::vector<Sprite>& out, std::string& err);
bool loadSprites(const std::string& path, std::vector<Sprite>& out, std::string& err);
const Sprite* findSprite(const std::vector<Sprite>& all, const std::string& name);
