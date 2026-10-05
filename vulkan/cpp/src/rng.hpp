#pragma once
#include <cstdint>

// xorshift32, the PRNG used for AI, effects and particles (seed 42 in bench/screenshots).
struct Rng {
    uint32_t s;
    explicit Rng(uint32_t seed = 42) : s(seed ? seed : 1u) {}
    uint32_t nextU32() {
        s ^= s << 13;
        s ^= s >> 17;
        s ^= s << 5;
        return s;
    }
    // Math.random() equivalent: [0, 1).
    double next() { return (nextU32() >> 8) * (1.0 / 16777216.0); }
};
