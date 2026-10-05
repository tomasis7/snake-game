#include "race.hpp"

#include <algorithm>
#include <cmath>
#include <cstdio>

RaceManager::RaceManager(const std::vector<int>& playerNumbers, double startX, double finishX)
    : startX_(startX), finishX_(finishX) {
    for (int pn : playerNumbers) headX_[pn] = startX;
}

void RaceManager::tick(double dtMs) {
    if (winner_ == 0) elapsed_ += dtMs;
}

double RaceManager::progress(int pn) const {
    double span = finishX_ - startX_;
    if (span <= 0) return 1;
    auto it = headX_.find(pn);
    double x = it == headX_.end() ? startX_ : it->second;
    return std::max(0.0, std::min(1.0, (x - startX_) / span));
}

void RaceManager::declareWinner(int pn, RaceReason reason) {
    if (winner_ != 0) return;
    winner_ = pn;
    reason_ = reason;
}

std::string formatTime(double ms) {
    double totalSeconds = ms / 1000;
    int m = (int)std::floor(totalSeconds / 60);
    int s = (int)std::floor(std::fmod(totalSeconds, 60.0));
    int tenths = (int)std::floor(std::fmod(totalSeconds * 10, 10.0));
    char buf[48];
    std::snprintf(buf, sizeof buf, "%d:%02d.%d", m, s, tenths);
    return buf;
}
