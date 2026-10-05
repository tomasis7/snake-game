#include "pathfinding.hpp"

#include <algorithm>
#include <cstdlib>

namespace {

const Dir DIRS[4] = {{1, 0}, {0, -1}, {0, 1}, {-1, 0}};

bool inBounds(const AIWorld& w, GridPos p) {
    return p.col >= w.minCol && p.col <= w.maxCol && p.row >= w.minRow && p.row <= w.maxRow;
}
bool isFree(const AIWorld& w, GridPos p) {
    return inBounds(w, p) && !w.blocked.count(cellKey(p.col, p.row));
}
bool isReverse(Dir a, Dir b) {
    return a.dx == -b.dx && a.dy == -b.dy && (a.dx != 0 || a.dy != 0);
}

}  // namespace

int64_t cellKey(int col, int row) { return ((int64_t)col << 32) ^ (int64_t)(uint32_t)row; }

std::optional<Dir> bfsFirstStep(const AIWorld& world, GridPos start, GridPos target, Dir currentDir) {
    int64_t targetKey = cellKey(target.col, target.row);
    if (cellKey(start.col, start.row) == targetKey) return std::nullopt;
    if (!isFree(world, target)) return std::nullopt;

    struct Node {
        GridPos pos;
        Dir first;
    };
    std::unordered_set<int64_t> visited{cellKey(start.col, start.row)};
    std::vector<Node> queue;

    for (const Dir& d : DIRS) {
        if (isReverse(d, currentDir)) continue;
        GridPos next{start.col + d.dx, start.row + d.dy};
        int64_t k = cellKey(next.col, next.row);
        if (k == targetKey) return d;
        if (!isFree(world, next)) continue;
        visited.insert(k);
        queue.push_back({next, d});
    }

    size_t head = 0;
    while (head < queue.size()) {
        Node n = queue[head++];
        for (const Dir& d : DIRS) {
            GridPos next{n.pos.col + d.dx, n.pos.row + d.dy};
            int64_t k = cellKey(next.col, next.row);
            if (visited.count(k)) continue;
            if (k == targetKey) return n.first;
            if (!isFree(world, next)) continue;
            visited.insert(k);
            queue.push_back({next, n.first});
        }
    }
    return std::nullopt;
}

Dir fallbackDir(const AIWorld& world, GridPos start, Dir currentDir) {
    for (const Dir& d : DIRS) {
        if (isReverse(d, currentDir)) continue;
        if (isFree(world, {start.col + d.dx, start.row + d.dy})) return d;
    }
    return currentDir;
}

Dir decideDirection(const AIWorld& world, GridPos start, Dir currentDir,
                    const std::vector<Pickup>& pickups, std::optional<GridPos> rivalHead) {
    struct Scored {
        GridPos pos;
        double score;
    };
    std::vector<Scored> scored;
    for (const Pickup& p : pickups) {
        if (p.pos.col < start.col - 2) continue;
        double value = p.value;
        if (rivalHead) {
            int rivalDist = std::abs(p.pos.col - rivalHead->col) + std::abs(p.pos.row - rivalHead->row);
            if (rivalDist <= 8) value *= 2;
        }
        int dist = std::abs(p.pos.col - start.col) + std::abs(p.pos.row - start.row);
        scored.push_back({p.pos, dist / value});
    }
    std::stable_sort(scored.begin(), scored.end(),
                     [](const Scored& a, const Scored& b) { return a.score < b.score; });

    for (size_t i = 0; i < scored.size() && i < 4; ++i) {
        if (auto step = bfsFirstStep(world, start, scored[i].pos, currentDir)) return *step;
    }
    GridPos rightTarget{world.maxCol, start.row};
    if (auto step = bfsFirstStep(world, start, rightTarget, currentDir)) return *step;
    return fallbackDir(world, start, currentDir);
}
