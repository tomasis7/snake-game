#pragma once
#include <vector>

#include "color.hpp"
#include "entities.hpp"

struct KeyState {
    bool up = false, down = false, left = false, right = false;
};

class Player {
public:
    Player(Vec2 position, int playerNumber, Color fill, Color stroke);
    virtual ~Player() = default;

    // dt: the p5 deltaTime; now: game clock in ms.
    void update(double dt, double now, const KeyState& keys);
    void applyStun(double now, double durationMs) { stunnedUntil_ = now + durationMs; }
    bool stunned(double now) const { return now < stunnedUntil_; }

    // 0 just stepped .. 1 about to step; accMs is the leftover fixed-step time.
    double moveProgress(double accMs, double now) const;
    Vec2 renderPos(size_t i, double accMs, double now) const;
    Vec2 interpolatedHead(double accMs, double now) const { return renderPos(0, accMs, now); }

    int playerNumber() const { return playerNumber_; }

    std::vector<Vec2> trail;
    Vec2 position;
    Vec2 direction{32, 0};
    Vec2 nextDirection{32, 0};
    Vec2 size{32, 32};
    Color fill, stroke;

    int lives = 3;
    int maxLives = 10;
    double lastCollisionTime = -1e9;  // game clock; far in the past so no start-up blink
    double collisionCooldown = 1000;
    bool canPassThroughObstacles = false;
    bool isColliding = false;

protected:
    virtual void handleInput(const KeyState& keys, double dt);

private:
    std::vector<Vec2> prevTrail_;
    int playerNumber_;
    double moveTimer_ = 0;
    double stunnedUntil_ = 0;
};
