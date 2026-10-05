#pragma once
#include <string>
#include <vector>

#include "entities.hpp"

constexpr int GRID_SIZE = 32;
constexpr int LEVEL_COUNT = 3;

struct LevelConfig {
    std::vector<std::string> layout;  // one string per row, one digit per tile
    double scrollSpeed = 0;
    double robotMistakeChance = 0;
};

// Reads a level file (one line per row). Returns empty on failure.
std::vector<std::string> loadLevelRows(const std::string& path);
LevelConfig getLevelConfig(int levelNumber, const std::string& assetDir);
// Tile (col,row) becomes an entity at (col*32+16, row*32+16); codes 1..7, 0 = nothing.
std::vector<Entity> createEntitiesForLevel(const std::vector<std::string>& layout);
double robotMistakeChanceFor(int levelNumber);
