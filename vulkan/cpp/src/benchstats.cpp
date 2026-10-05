#include "benchstats.hpp"

#include <algorithm>
#include <cmath>
#include <functional>
#include <numeric>

double round2(double v) { return std::round(v * 100.0) / 100.0; }

BenchStats computeBenchStats(const std::vector<double>& frameMs) {
    BenchStats r;
    size_t n = frameMs.size();
    r.frames = n;
    if (n == 0) return r;
    double sum = std::accumulate(frameMs.begin(), frameMs.end(), 0.0);
    r.avgMs = sum / (double)n;
    r.avgFps = sum > 0 ? (double)n / (sum / 1000.0) : 0;

    std::vector<double> sorted = frameMs;
    std::sort(sorted.begin(), sorted.end());
    size_t rank = (size_t)std::ceil(0.99 * (double)n);  // nearest-rank
    r.p99Ms = sorted[std::min(std::max<size_t>(rank, 1), n) - 1];

    size_t k = std::max<size_t>(1, n / 100);
    double slow = std::accumulate(sorted.end() - (long)k, sorted.end(), 0.0) / (double)k;
    r.p1LowFps = slow > 0 ? 1000.0 / slow : 0;
    return r;
}
