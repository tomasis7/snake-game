#version 450
// Instanced quad: no vertex buffer, 6 vertices per instance from gl_VertexIndex.
// Instances are in canvas pixels (1200 x 800 like the TS game, origin top-left).
// The camera transform is applied on the CPU; the viewport letterboxes the canvas.

layout(push_constant) uniform Push {
    vec2 canvasSize;  // 1200, 800
} pc;

layout(location = 0) in vec2 inPos;     // top-left corner, canvas pixels
layout(location = 1) in vec2 inSize;    // width, height, canvas pixels
layout(location = 2) in vec4 inColor;   // rgba 0..1 (tint for textured kinds)
layout(location = 3) in vec4 inUv;      // u0 v0 u1 v1 in the atlas, or a second colour
layout(location = 4) in vec4 inParams;  // x = kind, y/z/w = kind-specific

layout(location = 0) out vec4 vColor;
layout(location = 1) out vec2 vLocal;   // 0..1 inside the quad
layout(location = 2) out vec2 vUv;
layout(location = 3) flat out vec4 vExtra;
layout(location = 4) flat out vec4 vParams;
layout(location = 5) out vec2 vPixel;   // position inside the quad, canvas pixels

const vec2 CORNERS[6] = vec2[](
    vec2(0.0, 0.0), vec2(1.0, 0.0), vec2(1.0, 1.0),
    vec2(0.0, 0.0), vec2(1.0, 1.0), vec2(0.0, 1.0)
);

void main() {
    vec2 c = CORNERS[gl_VertexIndex];
    vec2 p = inPos + c * inSize;
    gl_Position = vec4(p / pc.canvasSize * 2.0 - 1.0, 0.0, 1.0);  // Vulkan NDC: y down
    vColor = inColor;
    vLocal = c;
    vUv = mix(inUv.xy, inUv.zw, c);
    vExtra = inUv;
    vParams = inParams;
    vPixel = c * inSize;
}
