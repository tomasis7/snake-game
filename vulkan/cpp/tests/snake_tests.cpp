#include <cstdio>

#include "game.hpp"

static int failures = 0;
#define CHECK(c) do { if (!(c)) { std::printf("FAIL %s:%d %s\n", __FILE__, __LINE__, #c); ++failures; } } while (0)

static void movement() {
    Game g;
    CHECK(g.snake().size() == 3);
    g.setState({{5, 5}, {4, 5}, {3, 5}}, Dir::Right, {0, 0});
    g.step();
    CHECK(g.snake().front() == (Cell{6, 5}));
    CHECK(g.snake().size() == 3);
    CHECK(g.snake().back() == (Cell{4, 5}));
}
static void growth() {
    Game g;
    g.setState({{5, 5}, {4, 5}, {3, 5}}, Dir::Right, {6, 5});
    g.step();
    CHECK(g.score() == 1);
    CHECK(g.snake().size() == 4);
    CHECK(g.tickSeconds() < 0.150);
    CHECK(!g.occupied(g.food()));
}
static void tick() {
    Game g;
    CHECK(g.tickSeconds() == 0.150);
    for (int i = 0; i < 30; ++i) {
        g.setState({{5, 5}, {4, 5}, {3, 5}}, Dir::Right, {6, 5});
        g.step();
    }
    CHECK(g.tickSeconds() == 0.060);
}
static void wall() {
    Game g;
    g.setState({{GRID - 1, 5}, {GRID - 2, 5}, {GRID - 3, 5}}, Dir::Right, {0, 0});
    g.step();
    CHECK(!g.alive());
}
static void selfHit() {
    Game g;
    g.setState({{5, 5}, {5, 6}, {4, 6}, {4, 5}, {4, 4}, {5, 4}, {6, 4}}, Dir::Up, {0, 0});
    g.step();  // head moves up into (5,4), which is body
    CHECK(!g.alive());
}
static void reversal() {
    Game g;
    CHECK(!g.queueTurn(Dir::Left));
    CHECK(g.queueTurn(Dir::Up));
    CHECK(!g.queueTurn(Dir::Down));  // reversal against last queued
    CHECK(g.queueTurn(Dir::Left));
    CHECK(!g.queueTurn(Dir::Down));  // queue full (max 2)
    g.step();
    CHECK(g.dir() == Dir::Up);
}
static void foodFree() {
    Game g(7);
    for (int i = 0; i < 5000; ++i) {
        if (!g.alive()) g.reset();
        g.queueTurn(autopilot(g));
        g.step();
        if (g.alive()) CHECK(!g.occupied(g.food()));
    }
}

int main() {
    movement(); growth(); tick(); wall(); selfHit(); reversal(); foodFree();
    if (failures) { std::printf("%d failures\n", failures); return 1; }
    std::printf("all tests passed\n");
    return 0;
}
