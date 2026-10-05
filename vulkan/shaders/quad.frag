#version 450
// Kinds (params.x):
//   0 SOLID    flat colour rect
//   1 TEXTURED atlas sample (nearest) * colour; uv = u0 v0 u1 v1
//   2 BALL     snake segment: radial gradient like the TS canvas gradient.
//              colour = mid stop (0.3), uv slot = edge colour (stop 1.0),
//              params.y = how far the centre stop is mixed toward white,
//              params.z = diameter in canvas pixels (for the antialiased rim).
//   3 SHADOW   soft round shadow: colour with alpha fading to 0 at the rim
//   4 ROUNDED  rounded rect, params.y = corner radius in canvas pixels

layout(set = 0, binding = 0) uniform sampler2D atlas;

layout(location = 0) in vec4 vColor;
layout(location = 1) in vec2 vLocal;
layout(location = 2) in vec2 vUv;
layout(location = 3) flat in vec4 vExtra;
layout(location = 4) flat in vec4 vParams;
layout(location = 5) in vec2 vPixel;

layout(location = 0) out vec4 outColor;

void main() {
    int kind = int(vParams.x + 0.5);

    if (kind == 1) {
        vec4 t = texture(atlas, vUv) * vColor;
        if (t.a < 0.01) discard;
        outColor = t;
        return;
    }

    if (kind == 2) {
        vec2 d = vLocal - 0.5;                       // diameter units, centre = 0
        if (dot(d, d) > 0.25) discard;               // outside the circle
        // Two-point gradient from (-0.3, -0.3) r=0.1 to (0, 0) r=0.8, approximated
        // by distance from the focal point.
        float t = clamp((length(d + vec2(0.3)) - 0.1) / 0.7, 0.0, 1.0);
        vec3 inner = mix(vColor.rgb, vec3(1.0), vParams.y);
        vec3 col = t < 0.3 ? mix(inner, vColor.rgb, t / 0.3)
                           : mix(vColor.rgb, vExtra.rgb, (t - 0.3) / 0.7);
        // 1 px antialiased rim
        float r = length(d) * 2.0;
        float px = 2.0 / max(vParams.z, 1.0);       // params.z = diameter in canvas px
        float a = 1.0 - smoothstep(1.0 - px, 1.0, r);
        outColor = vec4(col, vColor.a * a);
        return;
    }

    if (kind == 3) {
        float r = length(vLocal - 0.5) * 2.0;
        float a = 1.0 - smoothstep(0.4, 1.0, r);
        outColor = vec4(vColor.rgb, vColor.a * a);
        return;
    }

    if (kind == 4) {
        vec2 size = vPixel / max(vLocal, vec2(1e-4));
        vec2 half_ = size * 0.5;
        float rad = min(vParams.y, min(half_.x, half_.y));
        vec2 q = abs(vPixel - half_) - (half_ - rad);
        float dist = length(max(q, 0.0)) - rad;
        if (dist > 0.5) discard;
        outColor = vec4(vColor.rgb, vColor.a * clamp(0.5 - dist, 0.0, 1.0));
        return;
    }

    outColor = vColor;
}
