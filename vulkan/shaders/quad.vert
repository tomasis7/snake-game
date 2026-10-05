#version 450
// Instanced quad: no vertex buffer, 6 vertices per instance from gl_VertexIndex.
// Instance data is in normalized board space [0,1]^2 (origin top-left).

layout(location = 0) in vec2 inPos;    // top-left corner
layout(location = 1) in vec2 inSize;   // width, height
layout(location = 2) in vec4 inColor;  // rgba

layout(location = 0) out vec4 vColor;
layout(location = 1) out vec2 vLocal;  // 0..1 inside the quad

const vec2 CORNERS[6] = vec2[](
    vec2(0.0, 0.0), vec2(1.0, 0.0), vec2(1.0, 1.0),
    vec2(0.0, 0.0), vec2(1.0, 1.0), vec2(0.0, 1.0)
);

void main() {
    vec2 c = CORNERS[gl_VertexIndex];
    vec2 p = inPos + c * inSize;          // board space
    gl_Position = vec4(p * 2.0 - 1.0, 0.0, 1.0); // Vulkan NDC: y down
    vColor = inColor;
    vLocal = c;
}
