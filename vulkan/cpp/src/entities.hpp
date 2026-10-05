#pragma once

constexpr double CANVAS_W = 1200.0;
constexpr double CANVAS_H = 800.0;

struct Vec2 {
    double x = 0, y = 0;
};

// Codes match the TS level digits (1..7).
enum class EntityKind { Block = 1, Star, Heart, Plant, Ghost, Tetris, Win };

struct Entity {
    EntityKind kind = EntityKind::Block;
    // position is the sprite centre when drawing, but the top-left when colliding (as in the TS).
    double x = 0, y = 0, w = 32, h = 32;
    double vx = 0, vy = 0;
    bool removed = false;
    bool soundPlaying = false;
    double pulse = 1.0;  // heart pulse scale

    void update(double nowMs);
};

Entity makeEntity(EntityKind kind, double x, double y);
