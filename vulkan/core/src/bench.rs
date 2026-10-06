//! Bench helpers shared by the native and web apps: stress quads and the JSON report.

use crate::draw::{Color, Painter};

fn fract(v: f32) -> f32 {
    v - v.floor()
}

/// Bench stress quads (see SPEC.md), written after the scene.
pub fn stress_quads(p: &mut Painter, n: usize, t: f32) {
    for i in 0..n {
        let fi = i as f32;
        let x = fract(fi * 0.618_034 + t * 0.10) * 1200.0;
        let y = fract(fi * 0.754_877_7 + t * 0.07 + 0.05 * (t + fi * 0.001).sin()) * 800.0;
        let color: Color = [fract(fi * 0.13), fract(fi * 0.37), fract(fi * 0.71), 0.6];
        p.solid(x as f64, y as f64, 4.8, 4.8, color);
    }
}

/// The one-line JSON result. `extra` is spliced in after `present_mode` (for example `"api":"webgl2",`).
/// Returns `None` when no frames were measured.
pub fn report_json(
    implementation: &str,
    present_mode: &str,
    extra: &str,
    quads: usize,
    seconds: f64,
    frame_times: &[f64],
    gpu: &str,
) -> Option<String> {
    let mut v = frame_times.to_vec();
    let n = v.len();
    if n == 0 {
        return None;
    }
    let sum: f64 = v.iter().sum();
    let avg_ms = sum / n as f64;
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let p99_idx = (((0.99 * n as f64).ceil() as usize).max(1) - 1).min(n - 1);
    let worst = ((n as f64 * 0.01).floor() as usize).max(1);
    let worst_mean = v[n - worst..].iter().sum::<f64>() / worst as f64;
    Some(format!(
        "{{\"impl\":\"{}\",\"present_mode\":\"{}\",{}\"quads\":{},\"seconds\":{},\"frames\":{},\"avg_fps\":{:.2},\"p1_low_fps\":{:.2},\"avg_ms\":{:.2},\"p99_ms\":{:.2},\"gpu\":\"{}\"}}",
        implementation,
        present_mode,
        extra,
        quads,
        seconds,
        n,
        1000.0 / avg_ms,
        1000.0 / worst_mean,
        avg_ms,
        v[p99_idx],
        gpu.replace('\\', "\\\\").replace('"', "\\\"")
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_has_expected_fields() {
        let j = report_json("rust", "FIFO", "", 0, 1.0, &[10.0, 20.0], "gpu").unwrap();
        assert!(j.starts_with("{\"impl\":\"rust\",\"present_mode\":\"FIFO\",\"quads\":0"));
        assert!(j.contains("\"frames\":2"));
        assert!(report_json("x", "y", "", 0, 1.0, &[], "").is_none());
    }
}
