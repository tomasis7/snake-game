#include "levels.hpp"

#include <algorithm>
#include <cmath>
#include <fstream>

Entity makeEntity(EntityKind kind, double x, double y) {
    Entity e;
    e.kind = kind;
    e.x = x;
    e.y = y;
    switch (kind) {
        case EntityKind::Plant: e.w = 32; e.h = 64; break;
        case EntityKind::Ghost: e.w = 50; e.h = 50; e.vx = 0.3; e.vy = 0.3; break;
        default: e.w = 32; e.h = 32; break;
    }
    return e;
}

void Entity::update(double nowMs) {
    if (removed) return;
    if (kind == EntityKind::Heart) {
        pulse = 1 + 0.1 * std::sin(nowMs * 0.01);
    } else if (kind == EntityKind::Ghost) {
        x += vx;
        y += vy;
        if (x < 0 || x > CANVAS_W) vx *= -1;
        if (y < 0 || y > CANVAS_H) vy *= -1;
    }
}

std::vector<std::string> loadLevelRows(const std::string& path) {
    std::vector<std::string> rows;
    std::ifstream f(path);
    std::string line;
    while (std::getline(f, line)) {
        if (!line.empty() && line.back() == '\r') line.pop_back();
        if (!line.empty()) rows.push_back(line);
    }
    return rows;
}

double robotMistakeChanceFor(int levelNumber) {
    static const double chances[LEVEL_COUNT] = {0.25, 0.1, 0.0};
    int index = std::min(std::max(levelNumber, 1), LEVEL_COUNT);
    return chances[index - 1];
}

LevelConfig getLevelConfig(int levelNumber, const std::string& assetDir) {
    static const double scroll[LEVEL_COUNT] = {1.5, 2.0, 2.5};
    int index = std::min(std::max(levelNumber, 1), LEVEL_COUNT);
    LevelConfig c;
    c.layout = loadLevelRows(assetDir + "/levels/level" + std::to_string(index) + ".txt");
    c.scrollSpeed = scroll[index - 1];
    c.robotMistakeChance = robotMistakeChanceFor(index);
    return c;
}

std::vector<Entity> createEntitiesForLevel(const std::vector<std::string>& layout) {
    std::vector<Entity> out;
    for (size_t row = 0; row < layout.size(); ++row) {
        for (size_t col = 0; col < layout[row].size(); ++col) {
            int code = layout[row][col] - '0';
            if (code < 1 || code > 7) continue;
            out.push_back(makeEntity((EntityKind)code, col * GRID_SIZE + GRID_SIZE / 2.0,
                                     row * GRID_SIZE + GRID_SIZE / 2.0));
        }
    }
    return out;
}
