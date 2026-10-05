#include "player.hpp"

#include <algorithm>

Player::Player(Vec2 pos, int playerNumber, Color f, Color s)
    : fill(f), stroke(s), playerNumber_(playerNumber) {
    position = {pos.x + 16, pos.y + 16};
    for (int k = 1; k <= 8; ++k) trail.push_back({position.x - size.x * k, position.y});
    prevTrail_ = trail;
}

void Player::handleInput(const KeyState& k, double) {
    if (k.up && direction.y == 0) {
        nextDirection = {0, -32};
    } else if (k.down && direction.y == 0) {
        nextDirection = {0, 32};
    } else if (k.left && direction.x == 0) {
        nextDirection = {-32, 0};
    } else if (k.right && direction.x == 0) {
        nextDirection = {32, 0};
    }
}

void Player::update(double dt, double now, const KeyState& keys) {
    if (now < stunnedUntil_) return;
    moveTimer_ += dt;
    if (moveTimer_ >= 200) {
        moveTimer_ = -100;
        prevTrail_ = trail;
        direction = nextDirection;
        Vec2 head = trail[0];
        trail.insert(trail.begin(), Vec2{head.x + direction.x, head.y + direction.y});
        trail.pop_back();
    }
    handleInput(keys, dt);
}

double Player::moveProgress(double accMs, double now) const {
    double acc = now < stunnedUntil_ ? 0.0 : accMs;
    return std::max(0.0, std::min(1.0, (moveTimer_ + acc + 100) / 300));
}

Vec2 Player::renderPos(size_t i, double accMs, double now) const {
    double t = moveProgress(accMs, now);
    const Vec2& cur = trail[i];
    const Vec2& prev = i < prevTrail_.size() ? prevTrail_[i] : cur;
    return {prev.x + (cur.x - prev.x) * t, prev.y + (cur.y - prev.y) * t};
}
