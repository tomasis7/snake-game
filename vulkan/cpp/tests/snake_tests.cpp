// Unit tests: the TS vitest suites ported (camera, racemanager, progress, pathfinding, effects,
// sprites) plus level loading, player stepping and collision rules.
#include <algorithm>
#include <cmath>
#include <cstdio>
#include <filesystem>
#include <fstream>
#include <functional>
#include <string>
#include <vector>

#include "audio_iface.hpp"
#include "camera.hpp"
#include "collision.hpp"
#include "effects.hpp"
#include "levels.hpp"
#include "pathfinding.hpp"
#include "player.hpp"
#include "progress.hpp"
#include "race.hpp"
#include "rng.hpp"
#include "sprites.hpp"
#include "textlayout.hpp"

static int failures = 0;
static int checks = 0;
#define CHECK(c)                                                         \
    do {                                                                 \
        ++checks;                                                        \
        if (!(c)) {                                                      \
            std::printf("FAIL %s:%d  %s\n", __FILE__, __LINE__, #c);     \
            ++failures;                                                  \
        }                                                                \
    } while (0)
#define CHECK_NEAR(a, b, eps) CHECK(std::fabs((double)(a) - (double)(b)) <= (eps))

struct TestCase {
    const char* name;
    std::function<void()> fn;
};
static std::vector<TestCase>& registry() {
    static std::vector<TestCase> r;
    return r;
}
struct Reg {
    Reg(const char* n, std::function<void()> f) { registry().push_back({n, std::move(f)}); }
};
#define TEST(name)                          \
    static void name();                     \
    static Reg reg_##name(#name, name);     \
    static void name()

// ------------------------------------------------------------------ camera.test.ts

TEST(camera_centers_on_midpoint) {
    Camera c = fitCamera({100, 300}, {200, 200}, 1200, 800, 0, 0.1, 1);
    CHECK(c.centerX == 200);
    CHECK(c.centerY == 200);
}
TEST(camera_zooms_out_for_wide_gap) {
    Camera c = fitCamera({0, 2000}, {400, 400}, 1200, 800, 0, 0.1, 1);
    CHECK_NEAR(c.scale, 0.6, 1e-5);
}
TEST(camera_max_scale) {
    Camera c = fitCamera({500, 520}, {400, 400}, 1200, 800, 0, 0.5, 1);
    CHECK(c.scale == 1);
}
TEST(camera_min_scale) {
    Camera c = fitCamera({0, 100000}, {0, 0}, 1200, 800, 0, 0.4, 1);
    CHECK(c.scale == 0.4);
}
TEST(camera_padding) {
    Camera c = fitCamera({0, 1000}, {400, 400}, 1200, 800, 100, 0.1, 1);
    CHECK_NEAR(c.scale, 1, 1e-5);
    CHECK(c.centerX == 500);
}
TEST(visible_world_rect) {
    WorldRect r = visibleWorldRect({1, 1000, 500}, 1200, 800, 0);
    CHECK(r.left == 400 && r.right == 1600 && r.top == 100 && r.bottom == 900);
    WorldRect z = visibleWorldRect({0.5, 0, 0}, 1200, 800, 0);
    CHECK(z.left == -1200 && z.right == 1200 && z.top == -800 && z.bottom == 800);
    WorldRect m = visibleWorldRect({1, 0, 0}, 1200, 800, 50);
    CHECK(m.left == -650 && m.right == 650);
}
TEST(intersects_rect) {
    WorldRect rect{0, 100, 0, 100};
    CHECK(intersectsRect(40, 40, 20, 20, rect));
    CHECK(intersectsRect(90, 40, 20, 20, rect));
    CHECK(!intersectsRect(200, 40, 20, 20, rect));
    CHECK(!intersectsRect(40, -200, 20, 20, rect));
}
TEST(kill_line) {
    CHECK(advanceKillLine(0, 1500, 1000) == 500);
    CHECK(advanceKillLine(500, 1200, 1000) == 500);
    CHECK(advanceKillLine(0, 300, 1000) == 0);
}

// ------------------------------------------------------------------ racemanager.test.ts

TEST(race_progress) {
    RaceManager rm({1, 2}, 100, 1100);
    CHECK(rm.progress(1) == 0);
    rm.setHeadX(1, 1100);
    CHECK(rm.progress(1) == 1);
}
TEST(race_progress_clamps) {
    RaceManager rm({1}, 100, 1100);
    rm.setHeadX(1, 2000);
    CHECK(rm.progress(1) == 1);
    rm.setHeadX(1, 0);
    CHECK(rm.progress(1) == 0);
}
TEST(race_progress_fraction) {
    RaceManager rm({1}, 100, 1100);
    rm.setHeadX(1, 600);
    CHECK_NEAR(rm.progress(1), 0.5, 1e-5);
}
TEST(race_zero_width_course) {
    RaceManager rm({1}, 500, 500);
    CHECK(rm.progress(1) == 1);
}
TEST(race_clock) {
    RaceManager rm({1}, 0, 100);
    rm.tick(16);
    rm.tick(16);
    CHECK(rm.elapsedMs() == 32);
}
TEST(race_clock_freezes_on_winner) {
    RaceManager rm({1, 2}, 0, 100);
    rm.tick(100);
    rm.declareWinner(1, RaceReason::Finish);
    rm.tick(100);
    CHECK(rm.elapsedMs() == 100);
}
TEST(race_first_winner_kept) {
    RaceManager rm({1, 2}, 0, 100);
    rm.declareWinner(2, RaceReason::OpponentOut);
    rm.declareWinner(1, RaceReason::Finish);
    CHECK(rm.winner() == 2);
    CHECK(rm.winReason() == RaceReason::OpponentOut);
    CHECK(rm.isOver());
}
TEST(race_no_winner_initially) {
    RaceManager rm({1, 2}, 0, 100);
    CHECK(rm.winner() == 0);
    CHECK(!rm.isOver());
}
TEST(format_time) {
    CHECK(formatTime(0) == "0:00.0");
    CHECK(formatTime(65400) == "1:05.4");
    CHECK(formatTime(9900) == "0:09.9");
}

// ------------------------------------------------------------------ progress.test.ts

static std::string tempFile(const char* name) {
    auto p = std::filesystem::temp_directory_path() / (std::string("snake_vk_test_") + name);
    std::filesystem::remove(p);
    return p.string();
}

TEST(progress_start_run) {
    Progress p;
    p.startRun(GameMode::OnePlayer);
    p.currentLevel = 2;
    p.startRun(GameMode::TwoPlayer);
    CHECK(p.currentLevel == 1);
    CHECK(p.mode == GameMode::TwoPlayer);
}
TEST(progress_last_level) {
    Progress p;
    p.startRun(GameMode::OnePlayer);
    CHECK(!p.isLastLevel());
    p.currentLevel = 3;
    CHECK(p.isLastLevel());
}
TEST(progress_first_best) {
    Progress p;
    double v = 0;
    CHECK(!p.getBestTime(1, v));
    BestResult r = p.recordBestTime(1, 42000);
    CHECK(r.bestMs == 42000 && r.isNewBest);
    CHECK(p.getBestTime(1, v) && v == 42000);
}
TEST(progress_keeps_faster) {
    Progress p;
    p.recordBestTime(1, 42000);
    BestResult a = p.recordBestTime(1, 50000);
    CHECK(a.bestMs == 42000 && !a.isNewBest);
    BestResult b = p.recordBestTime(1, 30000);
    CHECK(b.bestMs == 30000 && b.isNewBest);
}
TEST(progress_per_level) {
    Progress p;
    p.recordBestTime(1, 40000);
    p.recordBestTime(2, 60000);
    double a = 0, b = 0;
    CHECK(p.getBestTime(1, a) && a == 40000);
    CHECK(p.getBestTime(2, b) && b == 60000);
}
TEST(progress_corrupted_file) {
    std::string path = tempFile("corrupt.txt");
    {
        std::ofstream f(path);
        f << "L1 garbage\nnonsense\nL2\n";
    }
    Progress p(path);
    double v = 0;
    CHECK(!p.getBestTime(1, v));
    BestResult r = p.recordBestTime(1, 12345);
    CHECK(r.bestMs == 12345 && r.isNewBest);
    std::filesystem::remove(path);
}
TEST(progress_missing_file_and_persistence) {
    std::string path = tempFile("persist/best_times.txt");
    {
        Progress p(path);  // missing file ignored
        double v = 0;
        CHECK(!p.getBestTime(1, v));
        p.recordBestTime(1, 40000);
        p.recordBestTime(3, 55500);
    }
    {
        std::ifstream f(path);
        std::string l1, l2;
        std::getline(f, l1);
        std::getline(f, l2);
        CHECK(l1 == "L1 40000");
        CHECK(l2 == "L3 55500");
    }
    Progress q(path);
    double v = 0;
    CHECK(q.getBestTime(3, v) && v == 55500);
    std::filesystem::remove_all(std::filesystem::path(path).parent_path());
}

// ------------------------------------------------------------------ ai/pathfinding.test.ts

static AIWorld makeWorld(std::vector<std::pair<int, int>> blocked = {}) {
    AIWorld w;
    for (auto& [c, r] : blocked) w.blocked.insert(cellKey(c, r));
    w.minCol = 0;
    w.maxCol = 20;
    w.minRow = 0;
    w.maxRow = 10;
    return w;
}
static const Dir RIGHT{1, 0};

TEST(bfs_steps_right) {
    auto step = bfsFirstStep(makeWorld(), {2, 5}, {8, 5}, RIGHT);
    CHECK(step && *step == (Dir{1, 0}));
}
TEST(bfs_routes_around_wall) {
    std::vector<std::pair<int, int>> wall;
    for (int r = 3; r <= 7; ++r) wall.push_back({4, r});
    auto step = bfsFirstStep(makeWorld(wall), {3, 5}, {8, 5}, RIGHT);
    CHECK(step.has_value());
    CHECK(step && step->dx == 0 && (step->dy == 1 || step->dy == -1));
}
TEST(bfs_unreachable) {
    auto step = bfsFirstStep(makeWorld({{4, 5}, {6, 5}, {5, 4}, {5, 6}}), {5, 5}, {10, 5}, RIGHT);
    CHECK(!step.has_value());
}
TEST(bfs_never_reverses) {
    auto step = bfsFirstStep(makeWorld(), {5, 5}, {2, 5}, RIGHT);
    CHECK(!(step && *step == (Dir{-1, 0})));
}
TEST(bfs_blocked_target) {
    auto step = bfsFirstStep(makeWorld({{8, 5}}), {5, 5}, {8, 5}, RIGHT);
    CHECK(!step.has_value());
}
TEST(fallback_prefers_right) { CHECK(fallbackDir(makeWorld(), {5, 5}, RIGHT) == (Dir{1, 0})); }
TEST(fallback_dodges) {
    Dir d = fallbackDir(makeWorld({{6, 5}}), {5, 5}, RIGHT);
    CHECK(d.dx == 0 && std::abs(d.dy) == 1);
}
TEST(fallback_boxed_in) {
    CHECK(fallbackDir(makeWorld({{4, 5}, {6, 5}, {5, 4}, {5, 6}}), {5, 5}, RIGHT) == RIGHT);
}
TEST(decide_heads_to_pickup) {
    Dir d = decideDirection(makeWorld(), {2, 5}, RIGHT, {{{2, 2}, 3}});
    CHECK(d == (Dir{0, -1}));
}
TEST(decide_prefers_star) {
    Dir d = decideDirection(makeWorld(), {5, 5}, RIGHT, {{{5, 9}, 1}, {{11, 5}, 3}});
    CHECK(d == (Dir{1, 0}));
}
TEST(decide_contests_rival) {
    Dir d = decideDirection(makeWorld(), {10, 5}, RIGHT, {{{10, 1}, 1}, {{10, 9}, 1}}, GridPos{10, 10});
    CHECK(d == (Dir{0, 1}));
}
TEST(decide_ignores_pickups_behind) {
    Dir d = decideDirection(makeWorld(), {10, 5}, RIGHT, {{{2, 5}, 3}});
    CHECK(d == (Dir{1, 0}));
}

// ------------------------------------------------------------------ effects/effects.test.ts

TEST(fx_spawns_particles) {
    Rng rng;
    Effects fx(rng);
    fx.burst(100, 100, hex("#ffffff"), 12);
    CHECK(fx.particleCount() == 12);
}
TEST(fx_retires_particles) {
    Rng rng;
    Effects fx(rng);
    fx.burst(100, 100, hex("#ffffff"), 8);
    for (int i = 0; i < 40; ++i) fx.update(50);
    CHECK(fx.particleCount() == 0);
}
TEST(fx_particles_alive_midway) {
    Rng rng;
    Effects fx(rng);
    fx.burst(100, 100, hex("#ffffff"), 8);
    fx.update(100);
    CHECK(fx.particleCount() == 8);
}
TEST(fx_particle_cap) {
    Rng rng;
    Effects fx(rng);
    for (int i = 0; i < 40; ++i) fx.burst(10, 10, hex("#ffffff"), 20);
    CHECK(fx.particleCount() <= 240);
}
TEST(fx_clamps_huge_step) {
    Rng rng;
    Effects fx(rng);
    fx.burst(10, 10, hex("#ffffff"), 8);
    fx.update(100000);
    CHECK(fx.particleCount() == 8);
}
TEST(fx_shake_rest) {
    Rng rng;
    Effects fx(rng);
    Vec2 o = fx.shakeOffset();
    CHECK(o.x == 0 && o.y == 0);
}
TEST(fx_shake_within_intensity) {
    Rng rng;
    Effects fx(rng);
    fx.shake(10);
    Vec2 o = fx.shakeOffset();
    CHECK(std::fabs(o.x) <= 10 && std::fabs(o.y) <= 10);
}
TEST(fx_shake_decays) {
    Rng rng;
    Effects fx(rng);
    fx.shake(10);
    for (int i = 0; i < 40; ++i) fx.update(50);
    Vec2 o = fx.shakeOffset();
    CHECK(o.x == 0 && o.y == 0);
}
TEST(fx_stronger_shake_wins) {
    Rng rng;
    Effects fx(rng);
    fx.shake(4);
    fx.shake(12);
    fx.update(0);
    CHECK(std::fabs(fx.shakeOffset().x) <= 12);
}
TEST(fx_flash_fades) {
    Rng rng;
    Effects fx(rng);
    fx.flash(hex("#ffffff"));
    CHECK(fx.flashAlpha() > 0);
    for (int i = 0; i < 40; ++i) fx.update(50);
    CHECK(fx.flashAlpha() == 0);
}
TEST(fx_floating_text) {
    Rng rng;
    Effects fx(rng);
    fx.floatText(50, 50, "MINE!", hex("#ff00ff"));
    CHECK(fx.textCount() == 1);
    for (int i = 0; i < 40; ++i) fx.update(50);
    CHECK(fx.textCount() == 0);
}
TEST(fx_text_cap) {
    Rng rng;
    Effects fx(rng);
    for (int i = 0; i < 40; ++i) fx.floatText(10, 10, "x2", hex("#ffffff"));
    CHECK(fx.textCount() <= 12);
}

// ------------------------------------------------------------------ art/sprites.test.ts

static std::vector<Sprite> loadAllSprites() {
    std::vector<Sprite> s;
    std::string err;
    bool ok = loadSprites(std::string(ASSET_DIR) + "/sprites.txt", s, err);
    CHECK(ok);
    if (!ok) std::printf("  %s\n", err.c_str());
    return s;
}
TEST(sprites_expected_set) {
    auto all = loadAllSprites();
    std::vector<std::string> names;
    for (auto& s : all) names.push_back(s.name);
    std::sort(names.begin(), names.end());
    std::vector<std::string> want{"ghost", "heart", "plant", "star", "starBright", "tetris", "wall", "win"};
    CHECK(names == want);
}
TEST(sprites_rectangular_and_palette) {
    for (auto& s : loadAllSprites()) {
        CHECK(!s.rows.empty());
        for (auto& row : s.rows) {
            CHECK((int)row.size() == (int)s.rows[0].size());
            for (char c : row) CHECK(c == '.' || s.palette.count(c));
        }
        for (auto& [ch, col] : s.palette) {
            (void)ch;
            CHECK(col.size() == 7 && col[0] == '#' && col.find_first_not_of("0123456789abcdefABCDEF", 1) == std::string::npos);
        }
    }
}
TEST(sprites_star_twinkle_shares_grid) {
    auto all = loadAllSprites();
    const Sprite* a = findSprite(all, "star");
    const Sprite* b = findSprite(all, "starBright");
    CHECK(a && b);
    if (a && b) {
        CHECK(a->rows == b->rows);
        CHECK(a->palette != b->palette);
    }
}
TEST(sprites_parse_errors) {
    std::vector<Sprite> out;
    std::string err;
    CHECK(!parseSprites("sprite x 2 1\nA #ffffff\nrows\nAAA\nend\n", out, err));
    out.clear();
    CHECK(parseSprites("sprite x 2 1\nA #ffffff\nrows\nA.\nend\n", out, err));
    CHECK(out.size() == 1 && out[0].cols == 2);
}

// ------------------------------------------------------------------ level loading

struct LevelExpect {
    int level;
    int counts[8];  // index = tile code 1..7
};
TEST(level_tile_counts_and_win_column) {
    const LevelExpect expect[] = {
        {1, {0, 528, 10, 15, 11, 11, 322, 19}},
        {2, {0, 528, 10, 15, 13, 14, 375, 19}},
        {3, {0, 528, 16, 23, 16, 20, 513, 19}},
    };
    for (const auto& e : expect) {
        LevelConfig cfg = getLevelConfig(e.level, ASSET_DIR);
        CHECK(cfg.layout.size() == 25);
        auto ents = createEntitiesForLevel(cfg.layout);
        int counts[8] = {};
        double winMin = 1e9, winMax = -1;
        for (auto& en : ents) {
            counts[(int)en.kind]++;
            if (en.kind == EntityKind::Win) {
                winMin = std::min(winMin, en.x);
                winMax = std::max(winMax, en.x);
            }
        }
        for (int k = 1; k <= 7; ++k) {
            CHECK(counts[k] == e.counts[k]);
        }
        CHECK(winMin == 263 * 32 + 16 && winMax == 263 * 32 + 16);
    }
    CHECK(getLevelConfig(1, ASSET_DIR).robotMistakeChance == 0.25);
    CHECK(getLevelConfig(2, ASSET_DIR).robotMistakeChance == 0.1);
    CHECK(getLevelConfig(3, ASSET_DIR).robotMistakeChance == 0.0);
}
TEST(level_entity_geometry) {
    auto ents = createEntitiesForLevel({"01", "40"});
    CHECK(ents.size() == 2);
    CHECK(ents[0].kind == EntityKind::Block && ents[0].x == 48 && ents[0].y == 16);
    CHECK(ents[1].kind == EntityKind::Plant && ents[1].x == 16 && ents[1].y == 48);
    CHECK(ents[1].w == 32 && ents[1].h == 64);
    auto ghost = createEntitiesForLevel({"5"});
    CHECK(ghost[0].w == 50 && ghost[0].vx == 0.3);
}

// ------------------------------------------------------------------ player stepping

static Player makePlayer() { return Player({128, 192}, 1, hex("#00FFFF"), hex("green")); }

TEST(player_initial_layout) {
    Player p = makePlayer();
    CHECK(p.trail.size() == 8);
    CHECK(p.trail[0].x == 112 && p.trail[0].y == 208);
    CHECK(p.trail[7].x == -112);
    CHECK(p.lives == 3 && p.maxLives == 10);
}
TEST(player_steps_after_one_cycle) {
    Player p = makePlayer();
    KeyState none;
    double now = 0;
    for (int i = 0; i < 11; ++i, now += 16.667) p.update(16.667, now, none);
    CHECK(p.trail[0].x == 112);  // 11 ticks = 183 ms: no step yet
    p.update(16.667, now, none);
    now += 16.667;
    CHECK(p.trail[0].x == 144 && p.trail[0].y == 208);  // 12th tick crosses 200 ms
    CHECK(p.trail.size() == 8);
    // The timer restarts at -100, so the next step is one 300 ms cycle (18 ticks) later.
    for (int i = 0; i < 17; ++i, now += 16.667) p.update(16.667, now, none);
    CHECK(p.trail[0].x == 144);
    p.update(16.667, now, none);
    CHECK(p.trail[0].x == 176);
    CHECK_NEAR(p.moveProgress(0, now), (-100 + 100) / 300.0, 1e-9);
}
TEST(player_cannot_reverse_into_itself) {
    Player p = makePlayer();
    KeyState left;
    left.left = true;
    double now = 0;
    for (int i = 0; i < 40; ++i, now += 16.667) p.update(16.667, now, left);
    CHECK(p.direction.x == 32 && p.direction.y == 0);
    CHECK(p.trail[0].x > 112);
    // Turning is allowed: up, then the snake moves up and cannot reverse down.
    KeyState up;
    up.up = true;
    for (int i = 0; i < 40; ++i, now += 16.667) p.update(16.667, now, up);
    CHECK(p.direction.y == -32);
    KeyState down;
    down.down = true;
    for (int i = 0; i < 40; ++i, now += 16.667) p.update(16.667, now, down);
    CHECK(p.direction.y == -32 || p.direction.y == 0);
    CHECK(p.direction.y != 32);
}
TEST(player_stun_freezes_movement) {
    Player p = makePlayer();
    KeyState none;
    p.applyStun(0, 600);
    double now = 0;
    for (int i = 0; i < 30; ++i, now += 16.667) p.update(16.667, now, none);  // 500 ms
    CHECK(p.trail[0].x == 112);
    for (int i = 0; i < 20; ++i, now += 16.667) p.update(16.667, now, none);
    CHECK(p.trail[0].x > 112);
}

// ------------------------------------------------------------------ collision rules

struct CollisionFixture {
    Rng rng;
    Effects fx{rng};
    NullAudio audio;
    Player p1 = makePlayer();
    std::vector<Entity> entities;
    int finished = 0, eliminated = 0;
    CollisionManager cm{{&p1}, &entities, &fx, &audio, [this](int pn) { finished = pn; },
                        [this](int pn) { eliminated = pn; }};
    void put(EntityKind k) {  // entity whose top-left is the player's head cell
        entities.push_back(makeEntity(k, p1.trail[0].x, p1.trail[0].y));
    }
};

TEST(collision_hazard_costs_life_cooldown_and_stun) {
    CollisionFixture f;
    f.put(EntityKind::Block);
    f.cm.checkCollision(5000);
    CHECK(f.p1.lives == 2);
    CHECK(f.p1.lastCollisionTime == 5000);
    CHECK(f.p1.stunned(5001));
    CHECK(!f.p1.stunned(5600));
    CHECK(f.fx.particleCount() == 10);
    CHECK(f.fx.flashAlpha() > 0);
    // Within the 1000 ms cooldown the overlap costs nothing more.
    f.p1.isColliding = false;
    f.cm.checkCollision(5500);
    CHECK(f.p1.lives == 2);
    // After the cooldown it does.
    f.p1.isColliding = false;
    f.cm.checkCollision(6100);
    CHECK(f.p1.lives == 1);
    CHECK(f.eliminated == 0);
}
TEST(collision_all_hazard_kinds) {
    for (EntityKind k : {EntityKind::Block, EntityKind::Tetris, EntityKind::Plant, EntityKind::Ghost}) {
        CollisionFixture f;
        f.put(k);
        f.cm.checkCollision(5000);
        CHECK(f.p1.lives == 2);
    }
}
TEST(collision_last_life_eliminates) {
    CollisionFixture f;
    f.p1.lives = 1;
    f.put(EntityKind::Block);
    f.cm.checkCollision(5000);
    CHECK(f.p1.lives == 0);
    CHECK(f.eliminated == 1);
}
TEST(collision_heart_capped_at_ten) {
    CollisionFixture f;
    f.p1.lives = 10;
    f.put(EntityKind::Heart);
    f.cm.checkCollision(5000);
    CHECK(f.p1.lives == 10);
    CHECK(f.entities[0].removed);
    CollisionFixture g;
    g.put(EntityKind::Heart);
    g.cm.checkCollision(5000);
    CHECK(g.p1.lives == 4);
}
TEST(collision_star_removed) {
    CollisionFixture f;
    f.put(EntityKind::Star);
    f.cm.checkCollision(5000);
    CHECK(f.entities[0].removed);
    CHECK(f.p1.lives == 3);
}
TEST(collision_winblock_finishes) {
    CollisionFixture f;
    f.put(EntityKind::Win);
    f.cm.checkCollision(5000);
    CHECK(f.finished == 1);
    CHECK(f.p1.lives == 3);
}
TEST(collision_uses_position_as_top_left) {
    CollisionFixture f;
    // A block centred on the head cell would overlap, but position is the top-left: 40 px right is clear.
    f.entities.push_back(makeEntity(EntityKind::Block, f.p1.trail[0].x + 40, f.p1.trail[0].y));
    f.cm.checkCollision(5000);
    CHECK(f.p1.lives == 3);
    f.entities.push_back(makeEntity(EntityKind::Block, f.p1.trail[0].x + 31, f.p1.trail[0].y + 31));
    f.cm.checkCollision(5000);
    CHECK(f.p1.lives == 2);
}

// ------------------------------------------------------------------ text layout

TEST(text_layout_alignment) {
    std::vector<Glyph> g;
    layoutText("AB", 100, 50, 10, HAlign::Left, g);
    CHECK(g.size() == 2 && g[0].x == 100 && g[1].x == 110 && g[0].y == 45 && g[0].index == 'A' - 32);
    g.clear();
    layoutText("AB", 100, 50, 10, HAlign::Center, g);
    CHECK(g[0].x == 90);
    g.clear();
    layoutText("AB", 100, 50, 10, HAlign::Right, g);
    CHECK(g[0].x == 80);
    g.clear();
    layoutText("A B", 0, 0, 10, HAlign::Left, g);
    CHECK(g.size() == 2 && g[1].x == 20);
    g.clear();
    layoutText("A\nB", 0, 100, 8, HAlign::Left, g);
    CHECK(g.size() == 2 && g[1].y - g[0].y == 10);
}

int main() {
    for (auto& t : registry()) {
        int before = failures;
        t.fn();
        if (failures != before) std::printf("  ^ in test %s\n", t.name);
    }
    std::printf("%zu tests, %d checks, %d failures\n", registry().size(), checks, failures);
    return failures ? 1 : 0;
}
