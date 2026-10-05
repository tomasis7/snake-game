#include "progress.hpp"

#include <cerrno>
#include <cmath>
#include <cstdlib>
#include <filesystem>
#include <fstream>
#include <sstream>

Progress::Progress(std::string path) : path_(std::move(path)) { load(); }

std::string Progress::defaultPath() {
    std::filesystem::path base;
    const char* xdg = std::getenv("XDG_DATA_HOME");
    if (xdg && *xdg) {
        base = xdg;
    } else {
        const char* home = std::getenv("HOME");
        if (!home) return "";
        base = std::filesystem::path(home) / ".local" / "share";
    }
    return (base / "furious-snake-vulkan" / "best_times.txt").string();
}

void Progress::startRun(GameMode m) {
    mode = m;
    currentLevel = 1;
}

void Progress::load() {
    if (path_.empty()) return;
    best_.clear();
    std::ifstream f(path_);
    if (!f) return;
    std::string line;
    while (std::getline(f, line)) {
        std::istringstream ls(line);
        std::string tag, val;
        if (!(ls >> tag >> val) || tag.size() < 2 || tag[0] != 'L') continue;
        char* end = nullptr;
        long level = std::strtol(tag.c_str() + 1, &end, 10);
        if (*end != '\0' || level <= 0) continue;
        char* vend = nullptr;
        double ms = std::strtod(val.c_str(), &vend);
        if (vend == val.c_str() || *vend != '\0' || !std::isfinite(ms)) continue;
        best_[(int)level] = ms;
    }
}

void Progress::save() const {
    if (path_.empty()) return;
    std::error_code ec;
    std::filesystem::create_directories(std::filesystem::path(path_).parent_path(), ec);
    std::ofstream f(path_, std::ios::trunc);
    if (!f) return;
    for (auto& [level, ms] : best_) f << 'L' << level << ' ' << (long long)std::llround(ms) << '\n';
}

bool Progress::getBestTime(int level, double& out) const {
    auto it = best_.find(level);
    if (it == best_.end()) return false;
    out = it->second;
    return true;
}

BestResult Progress::recordBestTime(int level, double timeMs) {
    load();  // pick up times written by the other port
    double prev = 0;
    bool has = getBestTime(level, prev);
    bool isNew = !has || timeMs < prev;
    if (isNew) {
        best_[level] = timeMs;
        save();
    }
    return {isNew ? timeMs : prev, isNew};
}
