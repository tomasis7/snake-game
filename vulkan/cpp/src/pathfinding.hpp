#pragma once
#include <cstdint>
#include <optional>
#include <unordered_set>
#include <vector>

struct GridPos {
    int col = 0, row = 0;
};
struct Dir {
    int dx = 0, dy = 0;
    bool operator==(const Dir& o) const { return dx == o.dx && dy == o.dy; }
};
struct Pickup {
    GridPos pos;
    double value = 1;
};
struct AIWorld {
    std::unordered_set<int64_t> blocked;
    int minCol = 0, maxCol = 0, minRow = 0, maxRow = 0;
};

int64_t cellKey(int col, int row);

// Shortest path via BFS; returns the FIRST step, or nullopt if unreachable.
std::optional<Dir> bfsFirstStep(const AIWorld& world, GridPos start, GridPos target, Dir currentDir);
// Any non-lethal direction, preferring rightward; keeps going if boxed in.
Dir fallbackDir(const AIWorld& world, GridPos start, Dir currentDir);
Dir decideDirection(const AIWorld& world, GridPos start, Dir currentDir,
                    const std::vector<Pickup>& pickups, std::optional<GridPos> rivalHead = std::nullopt);
