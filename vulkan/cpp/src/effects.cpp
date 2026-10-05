#include "effects.hpp"

#include <algorithm>
#include <cmath>

namespace {
constexpr double PARTICLE_LIFE_MS = 600;
constexpr double TEXT_LIFE_MS = 1200;
constexpr double SHAKE_DURATION_MS = 350;
constexpr double FLASH_DURATION_MS = 250;
constexpr double GRAVITY = 0.0006;
constexpr size_t MAX_PARTICLES = 240;
constexpr size_t MAX_TEXTS = 12;
constexpr double MAX_STEP_MS = 100;
constexpr double PI = 3.14159265358979323846;
}  // namespace

double Effects::flashAlpha() const {
    return flashRemaining_ <= 0 ? 0 : (flashRemaining_ / FLASH_DURATION_MS) * 70;
}

void Effects::burst(double x, double y, Color color, int count) {
    for (int i = 0; i < count; ++i) {
        double angle = (PI * 2 * i) / count + rng_.next() * 0.4;
        double speed = 0.06 + rng_.next() * 0.12;
        particles_.push_back({x, y, std::cos(angle) * speed, std::sin(angle) * speed, PARTICLE_LIFE_MS,
                              PARTICLE_LIFE_MS, color, 3 + rng_.next() * 3});
    }
    if (particles_.size() > MAX_PARTICLES)
        particles_.erase(particles_.begin(), particles_.begin() + (particles_.size() - MAX_PARTICLES));
}

void Effects::shake(double intensity) {
    shakeIntensity_ = std::max(shakeIntensity_, intensity);
    shakeRemaining_ = std::max(shakeRemaining_, SHAKE_DURATION_MS);
}

void Effects::flash(Color color) {
    flashColor_ = color;
    flashRemaining_ = FLASH_DURATION_MS;
}

void Effects::floatText(double x, double y, const std::string& text, Color color) {
    texts_.push_back({x, y, text, color, TEXT_LIFE_MS, TEXT_LIFE_MS});
    if (texts_.size() > MAX_TEXTS) texts_.erase(texts_.begin(), texts_.begin() + (texts_.size() - MAX_TEXTS));
}

void Effects::update(double dtMs) {
    double step = std::min(dtMs, MAX_STEP_MS);
    for (auto& p : particles_) {
        p.x += p.vx * step;
        p.y += p.vy * step;
        p.vy += GRAVITY * step;
        p.life -= step;
    }
    particles_.erase(std::remove_if(particles_.begin(), particles_.end(), [](const Particle& p) { return p.life <= 0; }),
                     particles_.end());
    for (auto& t : texts_) {
        t.y -= 0.03 * step;
        t.life -= step;
    }
    texts_.erase(std::remove_if(texts_.begin(), texts_.end(), [](const FloatingText& t) { return t.life <= 0; }),
                 texts_.end());

    shakeRemaining_ = std::max(0.0, shakeRemaining_ - step);
    if (shakeRemaining_ == 0) shakeIntensity_ = 0;
    flashRemaining_ = std::max(0.0, flashRemaining_ - step);
}

Vec2 Effects::shakeOffset() {
    if (shakeRemaining_ <= 0) return {0, 0};
    double falloff = shakeRemaining_ / SHAKE_DURATION_MS;
    double amount = shakeIntensity_ * falloff;
    double x = (rng_.next() * 2 - 1) * amount;
    double y = (rng_.next() * 2 - 1) * amount;
    return {x, y};
}
