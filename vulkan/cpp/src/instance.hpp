#pragma once

// 64-byte per-quad instance, matching vulkan/shaders/quad.vert.
struct Instance {
    float pos[2];     // top-left, canvas pixels
    float size[2];
    float color[4];
    float uv[4];      // u0 v0 u1 v1, or the BALL edge colour
    float params[4];  // x = kind
};
static_assert(sizeof(Instance) == 64);

enum QuadKind { KIND_SOLID = 0, KIND_TEXTURED = 1, KIND_BALL = 2, KIND_SHADOW = 3, KIND_ROUNDED = 4 };
