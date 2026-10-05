#pragma once
#include <memory>
#include <optional>
#include <string>

#include "audio_iface.hpp"
#include "drawlist.hpp"
#include "progress.hpp"
#include "rng.hpp"

// Fixed simulation step: the p5 deltaTime at frameRate(60).
constexpr double SIM_DT_MS = 16.667;

enum Key {
    KEY_UP, KEY_DOWN, KEY_LEFT, KEY_RIGHT, KEY_W, KEY_A, KEY_S, KEY_D,
    KEY_ENTER, KEY_ESC, KEY_1, KEY_2, KEY_H, KEY_N, KEY_R, KEY_M, KEY_G, KEY_COUNT
};

// Platform-independent input snapshot. `pressed` is a latched edge (cleared by the caller once a
// simulation step has consumed it); mouse coordinates are canvas pixels.
struct Input {
    bool down[KEY_COUNT] = {};
    bool pressed[KEY_COUNT] = {};
    float mouseX = -1, mouseY = -1;
    bool mouseDown = false;
};

class Game;

class Screen {
public:
    virtual ~Screen() = default;
    virtual void update(Game& g) = 0;
    virtual void draw(Game& g, DrawList& dl, double accMs) = 0;
};

struct BestInfo {
    double bestMs = 0;
    bool isNewBest = false;
};

class Game {
public:
    Game(std::string assetDir, IAudio& audio, Progress& progress, uint32_t seed);

    // One fixed 60 Hz step. Advances the game clock afterwards.
    void update(const Input& in);
    void draw(DrawList& dl, double accMs);

    void changeScreen(std::unique_ptr<Screen> s) { pending_ = std::move(s); }
    // Applies a pending screen change right away (for use outside update()).
    void flush() {
        if (pending_) current_ = std::move(pending_);
    }
    void startRun(GameMode mode);
    void startLevel(int level);
    void openMenu();
    void openHowTo();
    void openRace(int level, GameMode mode, bool bothRobots);
    void openResults(int level, GameMode mode, int winner, double humanTimeMs, std::optional<BestInfo> best);

    // Bench: both snakes on the robot AI (mistake chance 0), race restarts on the same level when it ends.
    void setBenchMode(bool on) { bench_ = on; }
    bool bench() const { return bench_; }

    const std::string& assetDir() const { return assetDir_; }
    const Input& input() const { return in_; }
    IAudio& audio() { return audio_; }
    Progress& progress() { return progress_; }
    Rng& rng() { return rng_; }
    double now() const { return now_; }       // game clock, ms
    bool quitRequested() const { return quit_; }
    void requestQuit() { quit_ = true; }
    bool showGrid() const { return showGrid_; }

private:
    std::string assetDir_;
    IAudio& audio_;
    Progress& progress_;
    Rng rng_;
    Input in_;
    double now_ = 0;
    bool quit_ = false;
    bool showGrid_ = false;
    bool bench_ = false;
    std::unique_ptr<Screen> current_, pending_;
};
