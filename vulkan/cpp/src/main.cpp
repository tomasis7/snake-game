#include <GLFW/glfw3.h>

#include <algorithm>
#include <chrono>
#include <cmath>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <exception>
#include <string>
#include <vector>

#include "game.hpp"
#include "renderer.hpp"

namespace {

using Clock = std::chrono::steady_clock;

float srgb(float c) { return c <= 0.04045f ? c / 12.92f : std::pow((c + 0.055f) / 1.055f, 2.4f); }

struct Color {
    float r, g, b;
};
// Hex colours are sRGB; the swapchain is sRGB so the shader output must be linear.
Color hex(unsigned v) {
    return {srgb(((v >> 16) & 255) / 255.0f), srgb(((v >> 8) & 255) / 255.0f), srgb((v & 255) / 255.0f)};
}

constexpr uint32_t BASE_INSTANCES = 1024;  // 400 cells + up to 400 snake + food, with headroom

uint32_t put(Instance* out, uint32_t n, float x, float y, float w, float h, Color c, float a = 1.0f) {
    out[n] = {{x, y}, {w, h}, {c.r, c.g, c.b, a}};
    return n + 1;
}

uint32_t buildScene(Instance* out, const Game& g) {
    static const Color cellC = hex(0x1e1e2a), headC = hex(0x7CFC00), bodyC = hex(0x32CD32),
                       foodC = hex(0xFF4757), deadC = hex(0xc0392b);
    const float cs = 1.0f / GRID;
    const float inset = 0.05f * cs;
    const float sz = cs - 2.0f * inset;
    uint32_t n = 0;
    for (int y = 0; y < GRID; ++y)
        for (int x = 0; x < GRID; ++x) n = put(out, n, x * cs + inset, y * cs + inset, sz, sz, cellC);
    n = put(out, n, g.food().x * cs + inset, g.food().y * cs + inset, sz, sz, foodC);
    const auto& s = g.snake();
    for (size_t i = 0; i < s.size(); ++i) {
        Color c;
        if (!g.alive()) {
            c = deadC;
        } else if (i == 0) {
            c = headC;
        } else {
            float k = 1.0f - 0.4f * (float)i / (float)s.size();  // darker toward the tail
            c = {bodyC.r * k, bodyC.g * k, bodyC.b * k};
        }
        n = put(out, n, s[i].x * cs + inset, s[i].y * cs + inset, sz, sz, c);
    }
    return n;
}

float fractf(float v) { return v - std::floor(v); }

void writeStress(Instance* out, uint32_t n, uint32_t count, float t) {
    for (uint32_t k = 0; k < count; ++k) {
        float i = (float)k;
        float x = fractf(i * 0.6180339f + t * 0.10f);
        float y = fractf(i * 0.3819660f + t * 0.07f + 0.05f * std::sin(t + i * 0.001f));
        out[n + k] = {{x, y},
                      {0.004f, 0.004f},
                      {fractf(i * 0.13f), fractf(i * 0.37f), fractf(i * 0.71f), 0.6f}};
    }
}

struct Input {
    Game* game = nullptr;
    bool* paused = nullptr;
    bool restart = false;
};

void keyCb(GLFWwindow* w, int key, int, int action, int) {
    if (action != GLFW_PRESS) return;
    auto* in = static_cast<Input*>(glfwGetWindowUserPointer(w));
    switch (key) {
        case GLFW_KEY_UP: case GLFW_KEY_W: in->game->queueTurn(Dir::Up); break;
        case GLFW_KEY_DOWN: case GLFW_KEY_S: in->game->queueTurn(Dir::Down); break;
        case GLFW_KEY_LEFT: case GLFW_KEY_A: in->game->queueTurn(Dir::Left); break;
        case GLFW_KEY_RIGHT: case GLFW_KEY_D: in->game->queueTurn(Dir::Right); break;
        case GLFW_KEY_R: case GLFW_KEY_ENTER: in->restart = true; break;
        case GLFW_KEY_P: case GLFW_KEY_SPACE: *in->paused = !*in->paused; break;
        case GLFW_KEY_ESCAPE: glfwSetWindowShouldClose(w, GLFW_TRUE); break;
        default: break;
    }
}

struct Args {
    bool bench = false;
    uint32_t quads = 0;
    double seconds = 10.0;
};

Args parseArgs(int argc, char** argv) {
    Args a;
    for (int i = 1; i < argc; ++i) {
        if (!std::strcmp(argv[i], "--bench")) a.bench = true;
        else if (!std::strcmp(argv[i], "--quads") && i + 1 < argc) a.quads = (uint32_t)std::strtoul(argv[++i], nullptr, 10);
        else if (!std::strcmp(argv[i], "--seconds") && i + 1 < argc) a.seconds = std::strtod(argv[++i], nullptr);
        else std::fprintf(stderr, "ignoring unknown argument: %s\n", argv[i]);
    }
    return a;
}

double round2(double v) { return std::round(v * 100.0) / 100.0; }

void printBench(const Renderer& r, const Args& a, const std::vector<double>& ms) {
    size_t n = ms.size();
    double sum = 0;
    for (double v : ms) sum += v;
    std::vector<double> sorted = ms;
    std::sort(sorted.begin(), sorted.end());
    double avgMs = n ? sum / (double)n : 0.0;
    double avgFps = sum > 0 ? (double)n * 1000.0 / sum : 0.0;
    size_t idx = n ? (size_t)std::ceil(0.99 * (double)n) - 1 : 0;
    double p99 = n ? sorted[std::min(idx, n - 1)] : 0.0;
    size_t k = std::max<size_t>(1, n / 100);
    double low = 0;
    for (size_t i = 0; i < k && i < n; ++i) low += sorted[n - 1 - i];
    low = n ? low / (double)k : 0.0;
    double p1 = low > 0 ? 1000.0 / low : 0.0;
    std::printf("{\"impl\":\"cpp\",\"present_mode\":\"%s\",\"quads\":%u,\"seconds\":%g,\"frames\":%zu,"
                "\"avg_fps\":%.2f,\"p1_low_fps\":%.2f,\"avg_ms\":%.2f,\"p99_ms\":%.2f,\"gpu\":\"%s\"}\n",
                r.presentModeName(), a.quads, a.seconds, n, round2(avgFps), round2(p1), round2(avgMs),
                round2(p99), r.gpuName().c_str());
    std::fflush(stdout);
}

int run(int argc, char** argv) {
    Args args = parseArgs(argc, argv);

    if (!glfwInit()) {
        std::fprintf(stderr, "glfwInit failed\n");
        return 1;
    }
    if (!glfwVulkanSupported()) {
        std::fprintf(stderr, "Vulkan not supported by GLFW\n");
        return 1;
    }
    glfwWindowHint(GLFW_CLIENT_API, GLFW_NO_API);
    GLFWwindow* window = glfwCreateWindow(800, 800, "Vulkan Snake (C++)", nullptr, nullptr);
    if (!window) {
        std::fprintf(stderr, "window creation failed\n");
        glfwTerminate();
        return 1;
    }

    int rc = 0;
    {
        uint32_t quads = args.bench ? args.quads : 0;
        Renderer renderer(window, BASE_INSTANCES + quads, args.bench);

        Game game(42);
        bool paused = false;
        Input input{&game, &paused, false};
        glfwSetWindowUserPointer(window, &input);
        glfwSetKeyCallback(window, keyCb);

        const double warmup = 2.0;
        std::vector<double> frameMs;
        if (args.bench) frameMs.reserve(1 << 20);

        auto start = Clock::now();
        auto last = start;
        double acc = 0.0;
        double titleAcc = 0.0;
        uint32_t titleFrames = 0;
        double fps = 0.0;

        while (!glfwWindowShouldClose(window)) {
            glfwPollEvents();
            auto now = Clock::now();
            double dt = std::chrono::duration<double>(now - last).count();
            last = now;
            double elapsed = std::chrono::duration<double>(now - start).count();

            if (args.bench && elapsed >= warmup + args.seconds) break;

            if (input.restart) {
                game.reset();
                acc = 0;
                input.restart = false;
            }
            if (!paused) {
                acc += std::min(dt, 0.25);
                while (acc >= game.tickSeconds()) {
                    acc -= game.tickSeconds();
                    if (args.bench) game.queueTurn(autopilot(game));
                    game.step();
                    if (args.bench && !game.alive()) game.reset();
                }
            }

            titleAcc += dt;
            ++titleFrames;
            if (titleAcc >= 0.5) {
                fps = titleFrames / titleAcc;
                titleAcc = 0;
                titleFrames = 0;
                char buf[128];
                std::snprintf(buf, sizeof buf, "Vulkan Snake (C++) | Score %d | Best %d | FPS %.0f",
                              game.score(), game.best(), fps);
                glfwSetWindowTitle(window, buf);
            }

            Instance* out = renderer.beginFrame();
            if (!out) {  // minimised
                glfwWaitEventsTimeout(0.05);
                last = Clock::now();
                continue;
            }
            uint32_t n = buildScene(out, game);
            if (quads) writeStress(out, n, quads, (float)elapsed);
            renderer.endFrame(n + quads);

            if (args.bench && elapsed >= warmup) frameMs.push_back(dt * 1000.0);
        }

        if (args.bench) printBench(renderer, args, frameMs);
    }

    glfwDestroyWindow(window);
    glfwTerminate();
    return rc;
}

}  // namespace

int main(int argc, char** argv) {
    try {
        return run(argc, argv);
    } catch (const std::exception& e) {
        std::fprintf(stderr, "error: %s\n", e.what());
        return 1;
    }
}
