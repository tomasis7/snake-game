#include "game.hpp"

#include "screens.hpp"

Game::Game(std::string assetDir, IAudio& audio, Progress& progress, uint32_t seed)
    : assetDir_(std::move(assetDir)), audio_(audio), progress_(progress), rng_(seed) {
    current_ = std::make_unique<StartMenu>(false);
}

void Game::update(const Input& in) {
    in_ = in;
    if (in_.pressed[KEY_G]) showGrid_ = !showGrid_;
    current_->update(*this);
    if (pending_) current_ = std::move(pending_);
    now_ += SIM_DT_MS;
}

void Game::draw(DrawList& dl, double accMs) {
    current_->draw(*this, dl, accMs);
    if (showGrid_) {
        // main.ts drawDebugGrid: stroke(200, 0, 0, 100), 1 px lines every 32 px.
        Color c{200 / 255.0f, 0, 0, 100 / 255.0f};
        for (int x = 0; x <= 1200; x += 32) dl.solid(x - 0.5f, 0, 1, 800, c);
        for (int y = 0; y <= 800; y += 32) dl.solid(0, y - 0.5f, 1200, 1, c);
    }
}

void Game::startRun(GameMode mode) {
    progress_.startRun(mode);
    startLevel(1);
}

void Game::startLevel(int level) {
    progress_.currentLevel = level;
    GameMode mode = progress_.mode;
    changeScreen(std::make_unique<CountDown>(now_, [this, level, mode]() {
        changeScreen(std::make_unique<GameBoard>(*this, level, mode, false));
    }));
}

void Game::openMenu() { changeScreen(std::make_unique<StartMenu>(in_.mouseDown)); }
void Game::openHowTo() { changeScreen(std::make_unique<InteractionScreen>(in_.mouseDown)); }

void Game::openRace(int level, GameMode mode, bool bothRobots) {
    changeScreen(std::make_unique<GameBoard>(*this, level, mode, bothRobots));
}

void Game::openResults(int level, GameMode mode, int winner, double humanTimeMs,
                       std::optional<BestInfo> best) {
    changeScreen(std::make_unique<ResultsScreen>(*this, level, mode, winner, humanTimeMs, best));
}
