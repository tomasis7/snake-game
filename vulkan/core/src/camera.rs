//! Port of camera.ts: pure camera math for framing both racers.

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Camera {
    pub scale: f64,
    pub center_x: f64,
    pub center_y: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorldRect {
    pub left: f64,
    pub right: f64,
    pub top: f64,
    pub bottom: f64,
}

fn min_of(v: &[f64]) -> f64 {
    v.iter().cloned().fold(f64::INFINITY, f64::min)
}

fn max_of(v: &[f64]) -> f64 {
    v.iter().cloned().fold(f64::NEG_INFINITY, f64::max)
}

/// Frames all given points (plus padding) into the viewport, centred on their
/// midpoint, with the zoom clamped between min_scale and max_scale.
pub fn fit_camera(
    xs: &[f64],
    ys: &[f64],
    viewport_w: f64,
    viewport_h: f64,
    padding: f64,
    min_scale: f64,
    max_scale: f64,
) -> Camera {
    let min_x = min_of(xs) - padding;
    let max_x = max_of(xs) + padding;
    let min_y = min_of(ys) - padding;
    let max_y = max_of(ys) + padding;
    let box_w = (max_x - min_x).max(1.0);
    let box_h = (max_y - min_y).max(1.0);
    let scale = min_scale.max(max_scale.min((viewport_w / box_w).min(viewport_h / box_h)));
    Camera { scale, center_x: (min_x + max_x) / 2.0, center_y: (min_y + max_y) / 2.0 }
}

/// The kill line trails the leader by max_gap and only ever advances.
pub fn advance_kill_line(prev: f64, leader_x: f64, max_gap: f64) -> f64 {
    prev.max(leader_x - max_gap)
}

pub fn visible_world_rect(cam: &Camera, viewport_w: f64, viewport_h: f64, margin: f64) -> WorldRect {
    let half_w = viewport_w / 2.0 / cam.scale + margin;
    let half_h = viewport_h / 2.0 / cam.scale + margin;
    WorldRect {
        left: cam.center_x - half_w,
        right: cam.center_x + half_w,
        top: cam.center_y - half_h,
        bottom: cam.center_y + half_h,
    }
}

/// True when an axis-aligned box (x, y top-left) overlaps the rect at all.
pub fn intersects_rect(x: f64, y: f64, w: f64, h: f64, r: &WorldRect) -> bool {
    x + w >= r.left && x <= r.right && y + h >= r.top && y <= r.bottom
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) {
        assert!((a - b).abs() < 1e-5, "{a} vs {b}");
    }

    #[test]
    fn fit_centers_on_midpoint() {
        let c = fit_camera(&[100.0, 300.0], &[200.0, 200.0], 1200.0, 800.0, 0.0, 0.1, 1.0);
        assert_eq!(c.center_x, 200.0);
        assert_eq!(c.center_y, 200.0);
    }

    #[test]
    fn fit_zooms_out_for_wide_gap() {
        let c = fit_camera(&[0.0, 2000.0], &[400.0, 400.0], 1200.0, 800.0, 0.0, 0.1, 1.0);
        close(c.scale, 0.6);
    }

    #[test]
    fn fit_never_zooms_past_max() {
        let c = fit_camera(&[500.0, 520.0], &[400.0, 400.0], 1200.0, 800.0, 0.0, 0.5, 1.0);
        assert_eq!(c.scale, 1.0);
    }

    #[test]
    fn fit_never_zooms_past_min() {
        let c = fit_camera(&[0.0, 100000.0], &[0.0, 0.0], 1200.0, 800.0, 0.0, 0.4, 1.0);
        assert_eq!(c.scale, 0.4);
    }

    #[test]
    fn fit_accounts_for_padding() {
        let c = fit_camera(&[0.0, 1000.0], &[400.0, 400.0], 1200.0, 800.0, 100.0, 0.1, 1.0);
        close(c.scale, 1.0);
        assert_eq!(c.center_x, 500.0);
    }

    #[test]
    fn visible_rect_scale_one() {
        let r = visible_world_rect(&Camera { scale: 1.0, center_x: 1000.0, center_y: 500.0 }, 1200.0, 800.0, 0.0);
        assert_eq!(r, WorldRect { left: 400.0, right: 1600.0, top: 100.0, bottom: 900.0 });
    }

    #[test]
    fn visible_rect_zoomed_out() {
        let r = visible_world_rect(&Camera { scale: 0.5, center_x: 0.0, center_y: 0.0 }, 1200.0, 800.0, 0.0);
        assert_eq!((r.left, r.right, r.top, r.bottom), (-1200.0, 1200.0, -800.0, 800.0));
    }

    #[test]
    fn visible_rect_margin() {
        let r = visible_world_rect(&Camera { scale: 1.0, center_x: 0.0, center_y: 0.0 }, 1200.0, 800.0, 50.0);
        assert_eq!(r.left, -650.0);
        assert_eq!(r.right, 650.0);
    }

    #[test]
    fn intersects() {
        let r = WorldRect { left: 0.0, right: 100.0, top: 0.0, bottom: 100.0 };
        assert!(intersects_rect(40.0, 40.0, 20.0, 20.0, &r));
        assert!(intersects_rect(90.0, 40.0, 20.0, 20.0, &r));
        assert!(!intersects_rect(200.0, 40.0, 20.0, 20.0, &r));
        assert!(!intersects_rect(40.0, -200.0, 20.0, 20.0, &r));
    }

    #[test]
    fn kill_line() {
        assert_eq!(advance_kill_line(0.0, 1500.0, 1000.0), 500.0);
        assert_eq!(advance_kill_line(500.0, 1200.0, 1000.0), 500.0);
        assert_eq!(advance_kill_line(0.0, 300.0, 1000.0), 0.0);
    }
}
