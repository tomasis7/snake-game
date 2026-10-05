#include "robot.hpp"

#include <algorithm>
#include <cmath>

namespace {
constexpr int CELL = 32;

GridPos toGrid(double x, double y) { return {(int)std::floor(x / CELL), (int)std::floor(y / CELL)}; }
int sgn(double v) { return v > 0 ? 1 : (v < 0 ? -1 : 0); }
}  // namespace

void RobotPlayer::handleInput(const KeyState&, double dt) {
    if (!hasContext_) return;

    thinkTimer_ += dt;
    if (thinkTimer_ < 200) return;
    thinkTimer_ = 0;

    AIWorld world;
    std::vector<Pickup> pickups;
    buildWorld(world, pickups);
    GridPos head = toGrid(trail[0].x, trail[0].y);
    Dir currentDir{sgn(direction.x), sgn(direction.y)};

    // On a "mistake" tick the robot stops chasing pickups but still refuses lethal moves.
    if (rng_.next() < mistakeChance_) {
        Dir safe = fallbackDir(world, head, currentDir);
        nextDirection = {(double)safe.dx * CELL, (double)safe.dy * CELL};
        return;
    }

    // Contested targeting is the perfect-robot (mistake chance 0) behaviour only.
    std::optional<GridPos> rivalHead;
    if (mistakeChance_ == 0 && !context_.otherTrails.empty() && !context_.otherTrails[0]->empty()) {
        const Vec2& r = (*context_.otherTrails[0])[0];
        rivalHead = toGrid(r.x, r.y);
    }

    // Survival mode: close to the kill line, stop chasing pickups and push right.
    static const std::vector<Pickup> none;
    const std::vector<Pickup>& chasing = (trail[0].x - context_.cameraOffset >= 200) ? pickups : none;

    Dir next = decideDirection(world, head, currentDir, chasing, rivalHead);
    nextDirection = {(double)next.dx * CELL, (double)next.dy * CELL};
}

void RobotPlayer::buildWorld(AIWorld& world, std::vector<Pickup>& pickups) const {
    int minCol = std::max(0, (int)std::floor(context_.cameraOffset / CELL));
    int maxCol = minCol + (int)std::ceil(CANVAS_W / CELL) + 4;
    world.minCol = minCol;
    world.maxCol = maxCol;
    world.minRow = 0;
    world.maxRow = (int)std::floor(CANVAS_H / CELL) - 1;

    for (const Entity& e : *context_.entities) {
        if (e.removed) continue;
        GridPos g = toGrid(e.x, e.y);
        if (g.col < minCol - 2 || g.col > maxCol) continue;
        if (e.kind == EntityKind::Star) {
            pickups.push_back({g, 3});
        } else if (e.kind == EntityKind::Heart) {
            pickups.push_back({g, 1});
        } else if (e.kind != EntityKind::Win) {
            world.blocked.insert(cellKey(g.col, g.row));
        }
    }
    for (const auto* trailPtr : context_.otherTrails) {
        for (const Vec2& s : *trailPtr) {
            GridPos g = toGrid(s.x, s.y);
            world.blocked.insert(cellKey(g.col, g.row));
        }
    }
    for (size_t i = 1; i < trail.size(); ++i) {
        GridPos g = toGrid(trail[i].x, trail[i].y);
        world.blocked.insert(cellKey(g.col, g.row));
    }
}
