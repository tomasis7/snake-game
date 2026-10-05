#include "game.hpp"

#include <algorithm>
#include <cstdlib>

Game::Game(uint32_t seed) : rng_(seed) { reset(); }

void Game::reset() {
    snake_.clear();
    queue_.clear();
    int c = GRID / 2;
    snake_.push_back({c, c});
    snake_.push_back({c - 1, c});
    snake_.push_back({c - 2, c});
    dir_ = Dir::Right;
    alive_ = true;
    score_ = 0;
    placeFood();
}

bool Game::occupied(Cell c) const {
    return std::find(snake_.begin(), snake_.end(), c) != snake_.end();
}

void Game::placeFood() {
    std::vector<Cell> freeCells;
    for (int y = 0; y < GRID; ++y)
        for (int x = 0; x < GRID; ++x)
            if (!occupied({x, y})) freeCells.push_back({x, y});
    if (freeCells.empty()) return;
    food_ = freeCells[rng_.next() % freeCells.size()];
}

bool Game::queueTurn(Dir d) {
    if (!alive_ || queue_.size() >= 2) return false;
    Dir last = lastDir();
    if (d == last || opposite(d, last)) return false;
    queue_.push_back(d);
    return true;
}

double Game::tickSeconds() const { return std::max(60, 150 - 5 * score_) / 1000.0; }

static bool outside(Cell c) { return c.x < 0 || c.y < 0 || c.x >= GRID || c.y >= GRID; }

void Game::step() {
    if (!alive_) return;
    if (!queue_.empty()) {
        dir_ = queue_.front();
        queue_.pop_front();
    }
    Cell d = delta(dir_);
    Cell h = snake_.front();
    Cell n{h.x + d.x, h.y + d.y};
    if (outside(n)) {
        alive_ = false;
        return;
    }
    bool eating = (n == food_);
    // The tail cell is vacated this tick unless we are eating.
    for (size_t i = 0; i < snake_.size(); ++i) {
        if (!eating && i + 1 == snake_.size()) break;
        if (snake_[i] == n) {
            alive_ = false;
            return;
        }
    }
    snake_.push_front(n);
    if (!eating) {
        snake_.pop_back();
    } else {
        ++score_;
        best_ = std::max(best_, score_);
        placeFood();
    }
}

void Game::setState(std::deque<Cell> s, Dir d, Cell f) {
    snake_ = std::move(s);
    dir_ = d;
    food_ = f;
    queue_.clear();
    alive_ = true;
}

Dir autopilot(const Game& g) {
    static constexpr Dir all[4] = {Dir::Up, Dir::Down, Dir::Left, Dir::Right};
    const auto& s = g.snake();
    Cell h = s.front();
    Cell f = g.food();
    Dir cur = g.dir();
    Dir best = cur;
    int bestDist = 1 << 30;
    bool found = false;
    for (Dir d : all) {
        if (opposite(d, cur)) continue;
        Cell dl = delta(d);
        Cell n{h.x + dl.x, h.y + dl.y};
        if (outside(n)) continue;
        bool eating = (n == f);
        bool hit = false;
        for (size_t i = 0; i < s.size(); ++i) {
            if (!eating && i + 1 == s.size()) break;
            if (s[i] == n) { hit = true; break; }
        }
        if (hit) continue;
        int dist = std::abs(n.x - f.x) + std::abs(n.y - f.y);
        if (dist < bestDist) { bestDist = dist; best = d; found = true; }
    }
    return found ? best : cur;
}
