#include "screens.hpp"

#include <algorithm>
#include <cmath>
#include <cstdio>

namespace {

constexpr float W = 1200.0f, H = 800.0f;

// gameboard.ts constants
constexpr double MAX_GAP = 1000;
constexpr double WARN_MARGIN = 280;
constexpr double CAMERA_PADDING = 220;
constexpr double MIN_SCALE = 0.5;
constexpr double MAX_SCALE = 1.0;

const Color GREEN = hex("#45FF8C");
const Color WHITE = hex("#FFFFFF");
const Color CYAN = hex("#00FFFF");
const Color MAGENTA = hex("#FF00FF");
const Color GOLD = hex("#FDD03C");
const Color BLACK = hex("#000000");

}  // namespace

// ---------------------------------------------------------------- Button

void Button::draw(DrawList& dl, float textSize) const {
    dl.solidCentered(x, y, w, h, hex(bg));
    dl.text(text, x, y, textSize, HAlign::Center, hex(fg));
}

bool Button::isClicked(const Input& in) {
    if (!in.mouseDown) {
        pressHandled_ = false;
        return false;
    }
    if (pressHandled_) return false;
    bool inside = in.mouseX > x - w / 2 && in.mouseX < x + w / 2 && in.mouseY > y - h / 2 && in.mouseY < y + h / 2;
    if (inside) {
        pressHandled_ = true;
        return true;
    }
    return false;
}

// ---------------------------------------------------------------- StartMenu

StartMenu::StartMenu(bool md)
    : start_("Start Game", W / 2, H / 2 + 125, "#515151", 350, 50, "#45FF8C", md),
      one_("1 Player vs Robot", W / 2, H / 2 - 100, "#515151", 420, 50, "#00FFFF", md),
      two_("2 Players", W / 2, H / 2 - 25, "#515151", 420, 50, "#FF00FF", md),
      how_("How to play", W / 2, H - 100, "#515151", 380, 50, "#FFFFFF", md) {}

void StartMenu::update(Game& g) {
    const Input& in = g.input();
    if (one_.isClicked(in) || in.pressed[KEY_1]) selected_ = GameMode::OnePlayer;
    if (two_.isClicked(in) || in.pressed[KEY_2]) selected_ = GameMode::TwoPlayer;
    if (in.pressed[KEY_ESC]) {
        g.requestQuit();
        return;
    }
    if (start_.isClicked(in) || in.pressed[KEY_ENTER]) {
        if (!g.audio().musicPlaying()) g.audio().loopMusic();
        g.startRun(selected_);
        return;
    }
    if (how_.isClicked(in) || in.pressed[KEY_H]) g.openHowTo();
}

void StartMenu::draw(Game&, DrawList& dl, double) {
    dl.text("Furious Snake", W / 2, H / 4 - 100, 42, HAlign::Center, GREEN);
    one_.bg = selected_ == GameMode::OnePlayer ? "white" : "#515151";
    two_.bg = selected_ == GameMode::TwoPlayer ? "white" : "#515151";
    dl.text("SELECT MODE", W / 2, H / 4, 32, HAlign::Center, GREEN);
    start_.draw(dl, 32);
    one_.draw(dl, 32);
    two_.draw(dl, 32);
    how_.draw(dl, 32);
}

// ---------------------------------------------------------------- InteractionScreen

InteractionScreen::InteractionScreen(bool md) : back_("< Back", 150, 75, "#515151", 200, 50, "#FFFFFF", md) {}

void InteractionScreen::update(Game& g) {
    const Input& in = g.input();
    if (back_.isClicked(in) || in.pressed[KEY_ESC] || in.pressed[KEY_ENTER]) g.openMenu();
}

void InteractionScreen::draw(Game&, DrawList& dl, double) {
    const float cx = W / 2, cy = H / 2;
    dl.text("HOW TO PLAY", cx, cy - 300, 32, HAlign::Center, GREEN);
    dl.text("What to eat and what to avoid", cx, cy - 250, 28, HAlign::Center, WHITE);

    dl.solidCentered(cx, cy - 100, 1200, 250, hex("#515151"));

    // Left-aligned 24 px text with a 2 px black stroke.
    auto line = [&](const char* s, float x, float y, Color c) { dl.textOutlined(s, x, y, 24, HAlign::Left, c); };
    line("Power-ups", cx - 300, cy - 200, GREEN);
    dl.sprite(SPR_HEART, cx - 330, cy - 160, 25, 25);
    line("= +1 Life", cx - 300, cy - 160, WHITE);
    dl.sprite(SPR_STAR, cx - 330, cy - 110, 25, 25);
    line("= x2 Points", cx - 300, cy - 110, WHITE);

    line("Obstacles", cx + 100, cy - 200, GREEN);
    dl.sprite(SPR_PLANT, cx + 70, cy - 160, 21, 40);
    line("= -2 Life", cx + 100, cy - 160, WHITE);
    dl.sprite(SPR_GHOST, cx + 70, cy - 110, 35, 35);
    line("= -1 Life, -5 Points", cx + 100, cy - 110, WHITE);
    dl.sprite(SPR_TETRIS, cx + 70, cy - 60, 25, 25);
    line("= Game Over", cx + 100, cy - 60, WHITE);
    dl.sprite(SPR_WALL, cx + 70, cy - 10, 25, 25);
    line("= Game Over", cx + 100, cy - 10, WHITE);

    dl.text("Use the following keys on you keyboard\nto navigate your snake in the game", cx, cy + 80, 28,
            HAlign::Center, WHITE);
    dl.text("Player 1", cx - 200, cy + 160, 28, HAlign::Center, CYAN);
    dl.text("Player 2", cx + 200, cy + 160, 28, HAlign::Center, MAGENTA);

    // The TS uses unicode arrows the pixel font lacks; ASCII stand-ins here.
    auto square = [&](float x, float y) { dl.strokeRectCentered(x, y, 50, 50, 2, WHITE); };
    dl.text("^", cx - 200, cy + 220, 28, HAlign::Center, WHITE);
    dl.text("< v >", cx - 200, cy + 280, 28, HAlign::Center, WHITE);
    square(cx - 200, cy + 220);
    square(cx - 200, cy + 280);
    square(cx - 140, cy + 280);
    square(cx - 260, cy + 280);

    dl.text("W", cx + 200, cy + 220, 28, HAlign::Center, WHITE);
    dl.text("A S D", cx + 200, cy + 280, 28, HAlign::Center, WHITE);
    square(cx + 200, cy + 220);
    square(cx + 200, cy + 280);
    square(cx + 140, cy + 280);
    square(cx + 260, cy + 280);

    back_.draw(dl, 28);
}

// ---------------------------------------------------------------- CountDown

void CountDown::update(Game& g) {
    double deltaSeconds = (g.now() - last_) / 1000;
    last_ = g.now();
    if (!complete_ && value_ > 0) {
        value_ -= deltaSeconds;
        if (value_ <= 0) {
            value_ = 0;
            complete_ = true;
            callback_();
        }
    }
}

void CountDown::draw(Game&, DrawList& dl, double) {
    dl.text("GET READY", W / 2, H / 4, 32, HAlign::Center, GREEN);
    int display = (int)std::ceil(value_);
    if (display > 0) dl.text(std::to_string(display), W / 2, H / 3, 84, HAlign::Center, WHITE);
}

// ---------------------------------------------------------------- ResultsScreen

ResultsScreen::ResultsScreen(Game& g, int level, GameMode mode, int winner, double humanTimeMs,
                             std::optional<BestInfo> best)
    : level_(level), mode_(mode), winner_(winner), timeMs_(humanTimeMs), best_(best), final_(level >= 3),
      retry_("Retry", W / 2, H / 2 + 210, "#515151", 300, 50, "#FDD03C", g.input().mouseDown),
      menu_("Menu", W / 2, H / 2 + 280, "#515151", 300, 50, "#FFFFFF", g.input().mouseDown) {
    bool humanWon = mode == GameMode::OnePlayer ? winner == 1 : winner != 0;
    if (!final_ && humanWon)
        next_.emplace("Next Level", W / 2, H / 2 + 140, "#515151", 300, 50, "#45FF8C", g.input().mouseDown);
}

std::string ResultsScreen::winnerText() const {
    if (mode_ == GameMode::OnePlayer) return winner_ == 1 ? "YOU REACHED THE GOAL!" : "ROBOT WINS!";
    return "PLAYER " + std::to_string(winner_) + " WINS!";
}

void ResultsScreen::update(Game& g) {
    const Input& in = g.input();
    // Always poll every button so the held-press guards stay current.
    bool next = next_ && next_->isClicked(in);
    bool retry = retry_.isClicked(in);
    bool menu = menu_.isClicked(in);
    if (next_ && (next || in.pressed[KEY_N])) {
        g.startLevel(level_ + 1);
        return;
    }
    if (retry || in.pressed[KEY_R]) {
        g.startLevel(level_);
        return;
    }
    if (menu || in.pressed[KEY_M] || in.pressed[KEY_ESC]) g.openMenu();
}

void ResultsScreen::draw(Game&, DrawList& dl, double) {
    dl.text(final_ ? "FINAL RESULTS" : "LEVEL " + std::to_string(level_) + " COMPLETE", W / 2, H / 6, 28,
            HAlign::Center, GREEN);
    dl.text(winnerText(), W / 2, H / 6 + 90, 44, HAlign::Center, WHITE);
    dl.text("TIME  " + formatTime(timeMs_), W / 2, H / 2 - 20, 24, HAlign::Center, CYAN);
    float buttonText = 24;
    if (best_) {
        dl.text(best_->isNewBest ? "NEW BEST!  " + formatTime(best_->bestMs) : "BEST  " + formatTime(best_->bestMs),
                W / 2, H / 2 + 20, 20, HAlign::Center, GOLD);
        buttonText = 20;  // textSize left over from the line above, as in the TS
    }
    if (next_) next_->draw(dl, buttonText);
    retry_.draw(dl, buttonText);
    menu_.draw(dl, buttonText);
}

// ---------------------------------------------------------------- GameBoard

GameBoard::GameBoard(Game& g, int level, GameMode mode, bool bothRobots)
    : level_(level), mode_(mode), bothRobots_(bothRobots) {
    LevelConfig config = getLevelConfig(level, g.assetDir());
    Rng& rng = g.rng();
    effects_ = std::make_unique<Effects>(rng);

    // Bench / screenshots drive both snakes with a perfect robot.
    double mistake = bothRobots ? 0.0 : config.robotMistakeChance;
    if (bothRobots) {
        auto r1 = std::make_unique<RobotPlayer>(Vec2{128, 192}, 1, hex("#00FFFF"), hex("green"), 0.0, rng);
        robots_.push_back(r1.get());
        players_.push_back(std::move(r1));
    } else {
        players_.push_back(std::make_unique<Player>(Vec2{128, 192}, 1, hex("#00FFFF"), hex("green")));
    }
    if (bothRobots || mode == GameMode::OnePlayer) {
        auto r2 = std::make_unique<RobotPlayer>(Vec2{128, 576}, 2, hex("#FF00FF"), hex("orange"), mistake, rng);
        robots_.push_back(r2.get());
        players_.push_back(std::move(r2));
    } else {
        players_.push_back(std::make_unique<Player>(Vec2{128, 576}, 2, hex("#FF00FF"), hex("orange")));
    }

    entities_ = createEntitiesForLevel(config.layout);

    double finishX = CANVAS_W * 4;
    for (const Entity& e : entities_)
        if (e.kind == EntityKind::Win) {
            finishX = e.x;
            break;
        }
    double startX = players_[0]->trail[0].x;
    race_ = std::make_unique<RaceManager>(std::vector<int>{1, 2}, startX, finishX);

    std::vector<Player*> raw;
    for (auto& p : players_) raw.push_back(p.get());
    collisions_ = std::make_unique<CollisionManager>(
        raw, &entities_, effects_.get(), &g.audio(),
        [this](int pn) { resolveRace(*game_, pn, RaceReason::Finish); },
        [this](int pn) { resolveRace(*game_, pn == 1 ? 2 : 1, RaceReason::OpponentOut); });
}

double GameBoard::leaderX() const {
    double m = players_[0]->trail[0].x;
    for (auto& p : players_) m = std::max(m, p->trail[0].x);
    return m;
}

void GameBoard::resolveRace(Game& g, int winner, RaceReason reason) {
    if (levelEnded_) return;
    levelEnded_ = true;
    race_->declareWinner(winner, reason);

    if (g.bench()) {
        g.openRace(level_, mode_, true);
        return;
    }
    double humanTimeMs = race_->elapsedMs();
    std::optional<BestInfo> best;
    if (winner == 1 && reason == RaceReason::Finish) {
        BestResult r = g.progress().recordBestTime(level_, humanTimeMs);
        best = BestInfo{r.bestMs, r.isNewBest};
    }
    g.openResults(level_, mode_, winner, humanTimeMs, best);
}

void GameBoard::update(Game& g) {
    if (levelEnded_) return;
    game_ = &g;
    const Input& in = g.input();
    const double now = g.now();

    if (in.pressed[KEY_ESC]) {
        g.openMenu();
        levelEnded_ = true;
        return;
    }

    killLine_ = advanceKillLine(killLine_, leaderX(), MAX_GAP);

    KeyState arrows{in.down[KEY_UP], in.down[KEY_DOWN], in.down[KEY_LEFT], in.down[KEY_RIGHT]};
    KeyState wasd{in.down[KEY_W], in.down[KEY_S], in.down[KEY_A], in.down[KEY_D]};

    for (size_t i = 0; i < players_.size(); ++i) {
        Player& p = *players_[i];
        for (RobotPlayer* r : robots_) {
            if (r != &p) continue;
            RobotContext ctx;
            ctx.entities = &entities_;
            ctx.cameraOffset = killLine_;
            for (auto& o : players_)
                if (o.get() != &p) ctx.otherTrails.push_back(&o->trail);
            r->setContext(ctx);
        }
        p.update(SIM_DT_MS, now, i == 0 ? arrows : wasd);
    }

    for (auto& p : players_) {
        if (p->trail[0].x + p->size.x < killLine_) {
            resolveRace(g, p->playerNumber() == 1 ? 2 : 1, RaceReason::FellBehind);
            if (levelEnded_) return;
        }
    }

    for (Entity& e : entities_) e.update(now);
    // gameboard.ts also runs flyingGhost(): ghosts step twice per frame.
    for (Entity& e : entities_)
        if (e.kind == EntityKind::Ghost) e.update(now);

    collisions_->checkCollision(now);

    for (auto& p : players_) race_->setHeadX(p->playerNumber(), p->trail[0].x);
    race_->tick(SIM_DT_MS);
    effects_->update(SIM_DT_MS);
}

Camera GameBoard::camera(double accMs, double now) const {
    std::vector<double> xs, ys;
    for (auto& p : players_) {
        Vec2 h = p->interpolatedHead(accMs, now);
        xs.push_back(h.x);
        ys.push_back(h.y);
    }
    return fitCamera(xs, ys, CANVAS_W, CANVAS_H, CAMERA_PADDING, MIN_SCALE, MAX_SCALE);
}

void GameBoard::drawPlayer(DrawList& dl, const Player& p, const Camera& cam, Vec2 shake, double accMs,
                           double now, double drawNow) const {
    // Blink while the post-hit cooldown runs.
    double sinceHit = drawNow - p.lastCollisionTime;
    if (sinceHit < p.collisionCooldown && (long long)std::floor(sinceHit / 80) % 2 == 0) return;

    const float s = (float)cam.scale;
    const float d = (float)std::max(p.size.x, p.size.y) * s;
    const Color shadowColor{0, 0, 0, 0.3f};
    const Color headMid = hex("#FFA500"), headEdge = hex("#804600");
    const Color bodyEdge = lerpColor(p.stroke, BLACK, 0.7f);

    auto toScreen = [&](Vec2 w) {
        return Vec2{(w.x - cam.centerX) * cam.scale + CANVAS_W / 2 + shake.x,
                    (w.y - cam.centerY) * cam.scale + CANVAS_H / 2 + shake.y};
    };

    for (size_t i = 0; i < p.trail.size(); ++i) {
        Vec2 sp = toScreen(p.renderPos(i, accMs, now));
        // shadowBlur 15, offset (5, 5): a soft round quad about 1.6 x the diameter.
        dl.shadow((float)sp.x + 5, (float)sp.y + 5, d * 1.6f, shadowColor);
        if (i == 0) dl.ball((float)sp.x, (float)sp.y, d, headMid, headEdge, 0.8f);
        else dl.ball((float)sp.x, (float)sp.y, d, p.fill, bodyEdge, 0.2f);

        if (i == 0) {
            auto sign = [](double v) { return v > 0 ? 1.0f : (v < 0 ? -1.0f : 0.0f); };
            float fx = sign(p.direction.x), fy = sign(p.direction.y);
            float sx = fy, sy = fx;
            float eye = d * 0.13f, fo = d * 0.18f, so = d * 0.18f;
            for (float sg : {1.0f, -1.0f}) {
                float x = (float)sp.x + fx * fo + sx * so * sg;
                float y = (float)sp.y + fy * fo + sy * so * sg;
                dl.solidCentered(x, y, eye * 2, eye * 2, hex("#ffffff"));
                dl.solidCentered(x + fx * eye * 0.4f, y + fy * eye * 0.4f, eye, eye, hex("#101820"));
            }
        }
    }
}

void GameBoard::draw(Game& g, DrawList& dl, double accMs) {
    const double now = g.now();
    const double drawNow = now + accMs;
    Camera cam = camera(accMs, now);
    Vec2 shake = effects_->shakeOffset();
    const float s = (float)cam.scale;

    // Screen-space background with parallax; positive modulo so tiles always cover the viewport.
    double bgShift = std::fmod(std::fmod(cam.centerX * 0.25, 1415.0) + 1415.0, 1415.0);
    int numBackgrounds = (int)std::ceil(CANVAS_W / 1415.0) + 2;
    for (int i = 0; i < numBackgrounds; ++i)
        dl.background((float)(i * 1415 - bgShift + shake.x), (float)shake.y, 1415, 800);

    auto toX = [&](double wx) { return (float)((wx - cam.centerX) * cam.scale + CANVAS_W / 2 + shake.x); };
    auto toY = [&](double wy) { return (float)((wy - cam.centerY) * cam.scale + CANVAS_H / 2 + shake.y); };

    WorldRect view = visibleWorldRect(cam, CANVAS_W, CANVAS_H, 64);
    for (const Entity& e : entities_) {
        if (e.removed || !intersectsRect(e.x, e.y, e.w, e.h, view)) continue;
        float cx = toX(e.x), cy = toY(e.y);
        switch (e.kind) {
            case EntityKind::Block: dl.sprite(SPR_WALL, cx, cy, (float)e.w * s, (float)e.h * s); break;
            case EntityKind::Star: {
                bool a = std::fmod(drawNow, 800.0) < 400;
                dl.sprite(a ? SPR_STAR : SPR_STAR_BRIGHT, cx, cy, (float)e.w * s, (float)e.h * s);
                break;
            }
            case EntityKind::Heart:
                dl.sprite(SPR_HEART, cx, cy, (float)(e.w * e.pulse) * s, (float)(e.h * e.pulse) * s);
                break;
            case EntityKind::Plant: dl.sprite(SPR_PLANT, cx, cy, (float)e.w * s, (float)e.h * s); break;
            case EntityKind::Ghost: {
                double bob = std::sin(drawNow / 400) * 3;
                dl.sprite(SPR_GHOST, cx, toY(e.y + bob), (float)e.w * s, (float)e.h * s);
                break;
            }
            case EntityKind::Tetris: dl.sprite(SPR_TETRIS, cx, cy, (float)e.w * s, (float)e.h * s); break;
            case EntityKind::Win: dl.sprite(SPR_WIN, cx, cy, (float)e.w * s, (float)e.h * s); break;
        }
    }

    for (auto& p : players_) drawPlayer(dl, *p, cam, shake, accMs, now, drawNow);

    // effects.drawWorld
    for (const Particle& p : effects_->particles()) {
        float fade = (float)(p.life / p.maxLife);
        dl.solidCentered(toX(p.x), toY(p.y), (float)p.size * s, (float)p.size * s, withAlpha(p.color, fade));
    }
    for (const FloatingText& t : effects_->texts()) {
        float fade = (float)(t.life / t.maxLife);
        dl.text(t.text, toX(t.x), toY(t.y), 16 * s, HAlign::Center, withAlpha(t.color, fade));
    }

    // effects.drawOverlay
    float flash = (float)effects_->flashAlpha();
    if (flash > 0) dl.solid(0, 0, W, H, withAlpha(effects_->flashColor(), flash / 255.0f));

    // Flashing warning over any racer near the kill line.
    if (std::fmod(drawNow, 500.0) < 300) {
        for (auto& p : players_) {
            const Vec2& head = p->trail[0];
            if (head.x >= killLine_ + WARN_MARGIN) continue;
            float sx = (float)((head.x - cam.centerX) * cam.scale + CANVAS_W / 2);
            float sy = (float)((head.y - cam.centerY) * cam.scale + CANVAS_H / 2);
            dl.text("OUT OF TIME!", sx, sy - 40, 18, HAlign::Center, hex("#ff2d55"));
        }
    }

    // HUD (racemanager.ts draw)
    const float barX = 210, barW = W - barX - 60, barY = 30;
    dl.text("LVL " + std::to_string(level_) + "/" + std::to_string(LEVEL_COUNT), 20, barY, 14, HAlign::Left, GREEN);
    dl.rounded(barX, barY - 6, barW, 12, 6, hex("#2a2a2a"));
    dl.solid(barX + barW, barY - 10, 6, 20, GREEN);
    for (auto& p : players_) {
        float x = barX + barW * (float)race_->progress(p->playerNumber());
        dl.roundedCentered(x, barY, 12, 18, 3, p->playerNumber() == 1 ? CYAN : MAGENTA);
    }
    dl.text(formatTime(race_->elapsedMs()), W - 20, barY, 14, HAlign::Right, WHITE);
    float ly = barY + 18;
    for (auto& p : players_) {
        Color c = p->playerNumber() == 1 ? CYAN : MAGENTA;
        for (int i = 0; i < std::max(0, p->lives); ++i) dl.solid(20 + i * 14.0f, ly, 10, 10, c);
        ly += 16;
    }
}
