#pragma once
#include <cstdint>
#include <deque>
#include <vector>

constexpr int GRID = 20;

struct Cell {
    int x = 0, y = 0;
    bool operator==(const Cell&) const = default;
};

enum class Dir { Up, Down, Left, Right };

inline Cell delta(Dir d) {
    switch (d) {
        case Dir::Up: return {0, -1};
        case Dir::Down: return {0, 1};
        case Dir::Left: return {-1, 0};
        default: return {1, 0};
    }
}
inline bool opposite(Dir a, Dir b) {
    return (a == Dir::Up && b == Dir::Down) || (a == Dir::Down && b == Dir::Up) ||
           (a == Dir::Left && b == Dir::Right) || (a == Dir::Right && b == Dir::Left);
}

struct XorShift32 {
    uint32_t s;
    explicit XorShift32(uint32_t seed = 42) : s(seed ? seed : 1) {}
    uint32_t next() {
        s ^= s << 13;
        s ^= s >> 17;
        s ^= s << 5;
        return s;
    }
};

class Game {
public:
    explicit Game(uint32_t seed = 42);

    void reset();                 // keeps RNG state and best score
    bool queueTurn(Dir d);        // false if rejected
    void step();                  // one logic tick (no-op when dead)

    // snake().front() is the head
    const std::deque<Cell>& snake() const { return snake_; }
    Cell food() const { return food_; }
    Dir dir() const { return dir_; }
    bool alive() const { return alive_; }
    int score() const { return score_; }
    int best() const { return best_; }
    double tickSeconds() const;
    bool occupied(Cell c) const;
    Dir lastDir() const { return queue_.empty() ? dir_ : queue_.back(); }

    // Test hook
    void setState(std::deque<Cell> snake, Dir dir, Cell food);

private:
    void placeFood();

    XorShift32 rng_;
    std::deque<Cell> snake_;
    std::deque<Dir> queue_;
    Cell food_{};
    Dir dir_ = Dir::Right;
    bool alive_ = true;
    int score_ = 0, best_ = 0;
};

// Greedy autopilot used by the benchmark: steer toward food, avoid walls/body when possible.
Dir autopilot(const Game& g);
