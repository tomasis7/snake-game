#pragma once
#include <cstddef>
#include <vector>

struct BenchStats {
    size_t frames = 0;
    double avgFps = 0, p1LowFps = 0, avgMs = 0, p99Ms = 0;
};

// frameMs: wall-clock gaps between successive loop iterations, in milliseconds.
// avg_fps = frames / sum of frame times; p99 is nearest-rank; 1% low = 1000 / mean of the
// slowest max(1, n/100) frames.
BenchStats computeBenchStats(const std::vector<double>& frameMs);
double round2(double v);
