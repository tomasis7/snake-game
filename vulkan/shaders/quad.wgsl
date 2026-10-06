// Hand port of quad.vert + quad.frag for WebGPU / WebGL2 (same kinds, same maths).
// Instanced quad: no per-vertex data, 6 vertices per instance from vertex_index.
// Instances are in canvas pixels (1200 x 800, origin top-left). WebGPU has no push
// constants, so canvasSize lives in a 16-byte uniform buffer.
//
// Kinds (params.x):
//   0 SOLID    flat colour rect
//   1 TEXTURED atlas sample (nearest) * colour; uv = u0 v0 u1 v1
//   2 BALL     snake segment: radial gradient; colour = mid stop (0.3), uv slot = edge colour,
//              params.y = centre mix toward white, params.z = diameter in canvas px
//   3 SHADOW   soft round shadow
//   4 ROUNDED  rounded rect, params.y = corner radius in canvas px

struct Canvas {
    size: vec2<f32>,
    pad: vec2<f32>,
}

@group(0) @binding(0) var<uniform> canvas: Canvas;
@group(0) @binding(1) var atlas: texture_2d<f32>;
@group(0) @binding(2) var atlas_sampler: sampler;

struct VsIn {
    @builtin(vertex_index) vertex: u32,
    @location(0) pos: vec2<f32>,
    @location(1) size: vec2<f32>,
    @location(2) color: vec4<f32>,
    @location(3) uv: vec4<f32>,
    @location(4) params: vec4<f32>,
}

struct VsOut {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) local: vec2<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) @interpolate(flat) extra: vec4<f32>,
    @location(4) @interpolate(flat) params: vec4<f32>,
    @location(5) pixel: vec2<f32>,
}

@vertex
fn vs_main(in: VsIn) -> VsOut {
    var corners = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 0.0), vec2<f32>(1.0, 1.0),
        vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 1.0), vec2<f32>(0.0, 1.0)
    );
    let c = corners[in.vertex];
    let p = in.pos + c * in.size;
    var out: VsOut;
    // WebGPU NDC has y up, so flip y relative to the Vulkan shader.
    let ndc = p / canvas.size * 2.0 - 1.0;
    out.position = vec4<f32>(ndc.x, -ndc.y, 0.0, 1.0);
    out.color = in.color;
    out.local = c;
    out.uv = mix(in.uv.xy, in.uv.zw, c);
    out.extra = in.uv;
    out.params = in.params;
    out.pixel = c * in.size;
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    // Sampled up front so it stays in uniform control flow.
    let tex = textureSample(atlas, atlas_sampler, in.uv);
    let kind = i32(in.params.x + 0.5);

    if (kind == 1) {
        let t = tex * in.color;
        if (t.a < 0.01) {
            discard;
        }
        return t;
    }

    if (kind == 2) {
        let d = in.local - vec2<f32>(0.5);
        if (dot(d, d) > 0.25) {
            discard;
        }
        let t = clamp((length(d + vec2<f32>(0.3)) - 0.1) / 0.7, 0.0, 1.0);
        let inner = mix(in.color.rgb, vec3<f32>(1.0), in.params.y);
        var col: vec3<f32>;
        if (t < 0.3) {
            col = mix(inner, in.color.rgb, t / 0.3);
        } else {
            col = mix(in.color.rgb, in.extra.rgb, (t - 0.3) / 0.7);
        }
        let r = length(d) * 2.0;
        let px = 2.0 / max(in.params.z, 1.0);
        let a = 1.0 - smoothstep(1.0 - px, 1.0, r);
        return vec4<f32>(col, in.color.a * a);
    }

    if (kind == 3) {
        let r = length(in.local - vec2<f32>(0.5)) * 2.0;
        let a = 1.0 - smoothstep(0.4, 1.0, r);
        return vec4<f32>(in.color.rgb, in.color.a * a);
    }

    if (kind == 4) {
        let size = in.pixel / max(in.local, vec2<f32>(1e-4));
        let half_ = size * 0.5;
        let rad = min(in.params.y, min(half_.x, half_.y));
        let q = abs(in.pixel - half_) - (half_ - vec2<f32>(rad));
        let dist = length(max(q, vec2<f32>(0.0))) - rad;
        if (dist > 0.5) {
            discard;
        }
        return vec4<f32>(in.color.rgb, in.color.a * clamp(0.5 - dist, 0.0, 1.0));
    }

    return in.color;
}
