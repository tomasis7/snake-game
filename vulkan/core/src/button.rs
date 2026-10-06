//! Port of button.ts (logic only; drawing lives in screens.rs).

use crate::input::Input;

pub struct Button {
    pub text: &'static str,
    pub cx: f64,
    pub cy: f64,
    pub w: f64,
    pub h: f64,
    pub bg: u32,
    pub color: u32,
    press_handled: bool,
}

impl Button {
    #[allow(clippy::too_many_arguments)]
    pub fn new(text: &'static str, cx: f64, cy: f64, bg: u32, w: f64, h: f64, color: u32, input: &Input) -> Self {
        // A press already in progress when the button is created must not count.
        Button { text, cx, cy, w, h, bg, color, press_handled: input.mouse_down }
    }

    pub fn is_clicked(&mut self, input: &Input) -> bool {
        if !input.mouse_down {
            self.press_handled = false;
            return false;
        }
        if self.press_handled {
            return false;
        }
        let inside = input.mouse_x > self.cx - self.w / 2.0
            && input.mouse_x < self.cx + self.w / 2.0
            && input.mouse_y > self.cy - self.h / 2.0
            && input.mouse_y < self.cy + self.h / 2.0;
        if inside {
            self.press_handled = true;
            return true;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn press_guard() {
        let mut inp = Input::default();
        inp.mouse_x = 10.0;
        inp.mouse_y = 10.0;
        inp.mouse_down = true;
        let mut b = Button::new("x", 10.0, 10.0, 0, 20.0, 20.0, 0, &inp);
        assert!(!b.is_clicked(&inp));
        inp.mouse_down = false;
        assert!(!b.is_clicked(&inp));
        inp.mouse_down = true;
        assert!(b.is_clicked(&inp));
        assert!(!b.is_clicked(&inp));
    }
}
