#version 450

layout(location = 0) in vec4 vColor;
layout(location = 1) in vec2 vLocal;

layout(location = 0) out vec4 outColor;

void main() {
    // Soft rounded-corner look: darken toward the edges of each cell.
    vec2 d = abs(vLocal - 0.5) * 2.0;
    float edge = max(d.x, d.y);
    float shade = 1.0 - 0.25 * smoothstep(0.75, 1.0, edge);
    outColor = vec4(vColor.rgb * shade, vColor.a);
}
