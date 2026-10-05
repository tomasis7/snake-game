#pragma once
#include <string>
#include <vector>

#include "color.hpp"
#include "entities.hpp"
#include "rng.hpp"

struct Particle {
    double x, y, vx, vy, life, maxLife;
    Color color;
    double size;
};
struct FloatingText {
    double x, y;
    std::string text;
    Color color;
    double life, maxLife;
};

// Particle bursts, screen shake, screen flash, floating text.
class Effects {
public:
    explicit Effects(Rng& rng) : rng_(rng) {}

    size_t particleCount() const { return particles_.size(); }
    size_t textCount() const { return texts_.size(); }
    // 0..70 on the 0..255 alpha scale.
    double flashAlpha() const;
    Color flashColor() const { return flashColor_; }
    const std::vector<Particle>& particles() const { return particles_; }
    const std::vector<FloatingText>& texts() const { return texts_; }

    void burst(double x, double y, Color color, int count = 14);
    void shake(double intensity);
    void flash(Color color);
    void floatText(double x, double y, const std::string& text, Color color);
    void update(double dtMs);
    Vec2 shakeOffset();

private:
    Rng& rng_;
    std::vector<Particle> particles_;
    std::vector<FloatingText> texts_;
    double shakeIntensity_ = 0, shakeRemaining_ = 0, flashRemaining_ = 0;
    Color flashColor_{1, 1, 1, 1};
};
