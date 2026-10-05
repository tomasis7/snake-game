// Furious Snake on Vulkan (C++): window, input, main loops (interactive, bench, screenshot).
#include <GLFW/glfw3.h>
#include <stb_image_write.h>

#include <algorithm>
#include <chrono>
#include <cmath>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <memory>
#include <string>

#include "atlas.hpp"
#include "audio.hpp"
#include "benchstats.hpp"
#include "drawlist.hpp"
#include "game.hpp"
#include "levels.hpp"
#include "renderer.hpp"

namespace {

struct Args {
    bool bench = false;
    uint32_t quads = 0;
    double seconds = 10;
    int level = 1;
    std::string screenshot;
    std::string screen = "menu";
    double afterMs = 1500;
    bool mute = false;
};

bool parseArgs(int argc, char** argv, Args& a) {
    for (int i = 1; i < argc; ++i) {
        std::string s = argv[i];
        auto next = [&]() -> const char* { return i + 1 < argc ? argv[++i] : nullptr; };
        const char* v = nullptr;
        if (s == "--bench") a.bench = true;
        else if (s == "--mute") a.mute = true;
        else if (s == "--quads" && (v = next())) a.quads = (uint32_t)std::strtoul(v, nullptr, 10);
        else if (s == "--seconds" && (v = next())) a.seconds = std::atof(v);
        else if (s == "--level" && (v = next())) a.level = std::clamp(std::atoi(v), 1, LEVEL_COUNT);
        else if (s == "--screenshot" && (v = next())) a.screenshot = v;
        else if (s == "--screen" && (v = next())) a.screen = v;
        else if (s == "--after-ms" && (v = next())) a.afterMs = std::atof(v);
        else {
            std::fprintf(stderr, "bad argument: %s\n", s.c_str());
            return false;
        }
    }
    return true;
}

// ---- input

struct KeyMap {
    int glfw;
    Key key;
};
const KeyMap KEYS[] = {
    {GLFW_KEY_UP, KEY_UP}, {GLFW_KEY_DOWN, KEY_DOWN}, {GLFW_KEY_LEFT, KEY_LEFT}, {GLFW_KEY_RIGHT, KEY_RIGHT},
    {GLFW_KEY_W, KEY_W}, {GLFW_KEY_A, KEY_A}, {GLFW_KEY_S, KEY_S}, {GLFW_KEY_D, KEY_D},
    {GLFW_KEY_ENTER, KEY_ENTER}, {GLFW_KEY_KP_ENTER, KEY_ENTER}, {GLFW_KEY_ESCAPE, KEY_ESC},
    {GLFW_KEY_1, KEY_1}, {GLFW_KEY_2, KEY_2}, {GLFW_KEY_H, KEY_H}, {GLFW_KEY_N, KEY_N},
    {GLFW_KEY_R, KEY_R}, {GLFW_KEY_M, KEY_M}, {GLFW_KEY_G, KEY_G},
};

bool g_latched[KEY_COUNT] = {};

void onKey(GLFWwindow*, int key, int, int action, int) {
    if (action != GLFW_PRESS) return;
    for (const KeyMap& k : KEYS)
        if (k.glfw == key) g_latched[k.key] = true;
}

Input snapshot(GLFWwindow* w, const Renderer& r) {
    Input in;
    for (const KeyMap& k : KEYS)
        if (glfwGetKey(w, k.glfw) == GLFW_PRESS) in.down[k.key] = true;
    std::memcpy(in.pressed, g_latched, sizeof in.pressed);
    in.mouseDown = glfwGetMouseButton(w, GLFW_MOUSE_BUTTON_LEFT) == GLFW_PRESS;
    double cx, cy;
    glfwGetCursorPos(w, &cx, &cy);
    int ww, wh, fw, fh;
    glfwGetWindowSize(w, &ww, &wh);
    glfwGetFramebufferSize(w, &fw, &fh);
    if (ww > 0 && wh > 0) {
        IRect cr = r.canvasRect();
        double px = cx * fw / ww, py = cy * fh / wh;
        if (cr.w > 0) {
            in.mouseX = (float)((px - cr.x) / cr.w * 1200.0);
            in.mouseY = (float)((py - cr.y) / cr.h * 800.0);
        }
    }
    return in;
}

void clearLatched() { std::memset(g_latched, 0, sizeof g_latched); }

double wallSeconds() {
    using clock = std::chrono::steady_clock;
    static const auto t0 = clock::now();
    return std::chrono::duration<double>(clock::now() - t0).count();
}

float fract(float v) { return v - std::floor(v); }

// ---- modes

int runInteractive(GLFWwindow* win, Renderer& r, Game& game, const Atlas& atlas, uint32_t cap) {
    double last = wallSeconds(), acc = 0;
    double fpsStart = last;
    int fpsFrames = 0;
    while (!glfwWindowShouldClose(win) && !game.quitRequested()) {
        glfwPollEvents();
        double now = wallSeconds();
        acc += std::min(now - last, 0.1) * 1000.0;
        last = now;

        Input in = snapshot(win, r);
        int steps = 0;
        while (acc >= SIM_DT_MS) {
            game.update(in);
            std::memset(in.pressed, 0, sizeof in.pressed);  // an edge is seen by one step only
            acc -= SIM_DT_MS;
            ++steps;
        }
        if (steps > 0) clearLatched();

        Instance* buf = r.beginFrame();
        if (!buf) {
            glfwWaitEventsTimeout(0.05);
            continue;
        }
        DrawList dl(buf, cap, atlas);
        game.draw(dl, acc);
        r.endFrame(dl.count());

        ++fpsFrames;
        if (now - fpsStart >= 0.5) {
            char title[96];
            std::snprintf(title, sizeof title, "Furious Snake (C++) | FPS %.0f", fpsFrames / (now - fpsStart));
            glfwSetWindowTitle(win, title);
            fpsStart = now;
            fpsFrames = 0;
        }
    }
    return 0;
}

int runBench(Renderer& r, Game& game, const Atlas& atlas, uint32_t cap, const Args& args) {
    constexpr double WARMUP = 2.0;
    game.setBenchMode(true);
    game.openRace(args.level, GameMode::OnePlayer, true);
    game.flush();

    const double loopStart = wallSeconds();
    double prev = loopStart, acc = 0, measureStart = 0;
    bool measuring = false;
    std::vector<double> frameMs;
    frameMs.reserve(1 << 16);
    Input in;

    for (;;) {
        glfwPollEvents();
        double now = wallSeconds();
        double gap = now - prev;
        prev = now;
        double tSec = now - loopStart;
        if (measuring) {
            frameMs.push_back(gap * 1000.0);
            if (now - measureStart >= args.seconds) break;
        } else if (tSec >= WARMUP) {
            measuring = true;
            measureStart = now;
        }

        acc += std::min(gap, 0.1) * 1000.0;
        while (acc >= SIM_DT_MS) {
            game.update(in);
            acc -= SIM_DT_MS;
        }

        Instance* buf = r.beginFrame();
        if (!buf) continue;
        DrawList dl(buf, cap, atlas);
        game.draw(dl, acc);

        if (args.quads) {
            uint32_t n = args.quads;
            Instance* q = dl.reserve(n);
            const float t = (float)tSec;
            for (uint32_t i = 0; i < n; ++i) {
                const float fi = (float)i;
                Instance& o = q[i];
                o.pos[0] = fract(fi * 0.6180339f + t * 0.10f) * 1200.0f;
                o.pos[1] = fract(fi * 0.7548777f + t * 0.07f + 0.05f * std::sin(t + fi * 0.001f)) * 800.0f;
                o.size[0] = o.size[1] = 4.8f;
                o.color[0] = fract(fi * 0.13f);
                o.color[1] = fract(fi * 0.37f);
                o.color[2] = fract(fi * 0.71f);
                o.color[3] = 0.6f;
                o.uv[0] = o.uv[1] = o.uv[2] = o.uv[3] = 0;
                o.params[0] = KIND_SOLID;
                o.params[1] = o.params[2] = o.params[3] = 0;
            }
        }
        r.endFrame(dl.count());
    }

    BenchStats st = computeBenchStats(frameMs);
    std::printf("{\"impl\":\"cpp\",\"present_mode\":\"%s\",\"quads\":%u,\"seconds\":%g,\"frames\":%zu,"
                "\"avg_fps\":%.2f,\"p1_low_fps\":%.2f,\"avg_ms\":%.2f,\"p99_ms\":%.2f,\"gpu\":\"%s\"}\n",
                r.presentModeName(), args.quads, args.seconds, st.frames, round2(st.avgFps), round2(st.p1LowFps),
                round2(st.avgMs), round2(st.p99Ms), r.gpuName().c_str());
    return 0;
}

int runScreenshot(GLFWwindow* win, Renderer& r, Game& game, Progress& progress, const Atlas& atlas, uint32_t cap,
                  const Args& args) {
    const std::string& s = args.screen;
    if (s == "menu") {
    } else if (s == "howto") {
        game.openHowTo();
    } else if (s == "countdown") {
        progress.startRun(GameMode::OnePlayer);
        game.startLevel(1);
    } else if (s == "race") {
        game.openRace(1, GameMode::OnePlayer, true);
    } else if (s == "results") {
        game.openResults(1, GameMode::OnePlayer, 1, 42300, BestInfo{42300, true});
    } else {
        std::fprintf(stderr, "unknown screen: %s\n", s.c_str());
        return 2;
    }
    game.flush();

    // Let the compositor configure the window before the first frame.
    for (int i = 0; i < 5; ++i) glfwPollEvents();

    Input in;
    while (game.now() < args.afterMs) game.update(in);

    for (int attempt = 0; attempt < 120; ++attempt) {
        glfwPollEvents();
        Instance* buf = r.beginFrame();
        if (!buf) continue;
        DrawList dl(buf, cap, atlas);
        game.draw(dl, 0.0);
        Capture cap2;
        r.endFrame(dl.count(), &cap2);
        if (cap2.rgba.empty()) continue;
        if (!stbi_write_png(args.screenshot.c_str(), cap2.width, cap2.height, 4, cap2.rgba.data(), cap2.width * 4)) {
            std::fprintf(stderr, "cannot write %s\n", args.screenshot.c_str());
            return 1;
        }
        (void)win;
        return 0;
    }
    std::fprintf(stderr, "could not capture a frame\n");
    return 1;
}

}  // namespace

int main(int argc, char** argv) {
    Args args;
    if (!parseArgs(argc, argv, args)) return 2;
    const bool screenshot = !args.screenshot.empty();

    try {
        if (!glfwInit()) {
            std::fprintf(stderr, "glfwInit failed\n");
            return 1;
        }
        if (!glfwVulkanSupported()) {
            std::fprintf(stderr, "Vulkan not supported by GLFW\n");
            return 1;
        }
        glfwWindowHint(GLFW_CLIENT_API, GLFW_NO_API);
        glfwWindowHint(GLFW_RESIZABLE, GLFW_TRUE);
        GLFWwindow* win = glfwCreateWindow(1200, 800, "Furious Snake (C++)", nullptr, nullptr);
        if (!win) {
            std::fprintf(stderr, "window creation failed\n");
            return 1;
        }
        glfwSetKeyCallback(win, onKey);

        Atlas atlas = buildAtlas(ASSET_DIR);
        const uint32_t cap = args.quads + 65536;
        int rc = 0;
        {
            Renderer renderer(win, cap, args.bench, atlas);

            std::unique_ptr<Audio> audio;
            NullAudio nullAudio;
            IAudio* audioPtr = &nullAudio;
            if (!args.mute && !args.bench && !screenshot) {
                audio = Audio::create(PUBLIC_ASSET_DIR);
                if (audio) audioPtr = audio.get();
            }
            Progress progress(args.bench || screenshot ? "" : Progress::defaultPath());
            uint32_t seed = 42;
            if (!args.bench && !screenshot)
                seed = (uint32_t)std::chrono::steady_clock::now().time_since_epoch().count() | 1u;
            Game game(ASSET_DIR, *audioPtr, progress, seed);

            if (args.bench) rc = runBench(renderer, game, atlas, cap, args);
            else if (screenshot) rc = runScreenshot(win, renderer, game, progress, atlas, cap, args);
            else {
                audioPtr->loopMusic();  // main.ts setup(): music loops from the start
                rc = runInteractive(win, renderer, game, atlas, cap);
            }
        }
        glfwDestroyWindow(win);
        glfwTerminate();
        return rc;
    } catch (const std::exception& e) {
        std::fprintf(stderr, "error: %s\n", e.what());
        return 1;
    }
}
