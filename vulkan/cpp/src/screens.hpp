#pragma once
#include <functional>
#include <memory>
#include <optional>
#include <vector>

#include "camera.hpp"
#include "collision.hpp"
#include "effects.hpp"
#include "game.hpp"
#include "levels.hpp"
#include "player.hpp"
#include "race.hpp"
#include "robot.hpp"

// button.ts: centred rect + centred text; a press already held when created does not click.
class Button {
public:
    Button(std::string text, float x, float y, std::string bg, float w, float h, std::string fg, bool mouseDown)
        : text(std::move(text)), x(x), y(y), w(w), h(h), bg(std::move(bg)), fg(std::move(fg)), pressHandled_(mouseDown) {}
    void draw(DrawList& dl, float textSize) const;
    bool isClicked(const Input& in);

    std::string text;
    float x, y, w, h;
    std::string bg, fg;

private:
    bool pressHandled_;
};

class StartMenu : public Screen {
public:
    explicit StartMenu(bool mouseDown);
    void update(Game& g) override;
    void draw(Game& g, DrawList& dl, double accMs) override;

private:
    Button start_, one_, two_, how_;
    GameMode selected_ = GameMode::OnePlayer;
};

class InteractionScreen : public Screen {
public:
    explicit InteractionScreen(bool mouseDown);
    void update(Game& g) override;
    void draw(Game& g, DrawList& dl, double accMs) override;

private:
    Button back_;
};

class CountDown : public Screen {
public:
    CountDown(double nowMs, std::function<void()> callback)
        : last_(nowMs), callback_(std::move(callback)) {}
    void update(Game& g) override;
    void draw(Game& g, DrawList& dl, double accMs) override;

private:
    double value_ = 3;
    double last_;
    bool complete_ = false;
    std::function<void()> callback_;
};

class ResultsScreen : public Screen {
public:
    ResultsScreen(Game& g, int level, GameMode mode, int winner, double humanTimeMs, std::optional<BestInfo> best);
    void update(Game& g) override;
    void draw(Game& g, DrawList& dl, double accMs) override;

private:
    std::string winnerText() const;

    int level_;
    GameMode mode_;
    int winner_;
    double timeMs_;
    std::optional<BestInfo> best_;
    bool final_;
    std::optional<Button> next_;
    Button retry_, menu_;
};

class GameBoard : public Screen {
public:
    GameBoard(Game& g, int level, GameMode mode, bool bothRobots);
    void update(Game& g) override;
    void draw(Game& g, DrawList& dl, double accMs) override;

private:
    void resolveRace(Game& g, int winner, RaceReason reason);
    double leaderX() const;
    Camera camera(double accMs, double now) const;
    void drawPlayer(DrawList& dl, const Player& p, const Camera& cam, Vec2 shake, double accMs, double now,
                    double drawNow) const;

    int level_;
    GameMode mode_;
    bool bothRobots_;
    bool levelEnded_ = false;
    double killLine_ = 0;
    Game* game_ = nullptr;  // valid during update(), for the collision callbacks

    std::vector<Entity> entities_;
    std::vector<std::unique_ptr<Player>> players_;
    std::vector<RobotPlayer*> robots_;
    std::unique_ptr<Effects> effects_;
    std::unique_ptr<RaceManager> race_;
    std::unique_ptr<CollisionManager> collisions_;
};
