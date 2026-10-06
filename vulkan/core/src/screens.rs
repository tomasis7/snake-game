//! Menu, How to play, Countdown and Results screens (startmenu.ts, interactionscreen.ts,
//! countdown.ts, resultsscreen.ts).

use crate::button::Button;
use crate::draw::{rgb, Painter};
use crate::input::{Input, Tap};
use crate::progress::GameMode;
use crate::race::format_time;
use crate::sprites;
use crate::text::HAlign;

const W: f64 = 1200.0;
const H: f64 = 800.0;
const GREEN: u32 = 0x45FF8C;
const GREY: u32 = 0x515151;

pub enum Transition {
    ToMenu,
    ToHowTo,
    StartRun(GameMode),
    StartLevel(u32),
    BeginRace(u32),
    Quit,
}

fn draw_button(p: &mut Painter, b: &Button, text_size: f64) {
    p.solid_center(b.cx, b.cy, b.w, b.h, rgb(b.bg));
    p.text(b.text, b.cx, b.cy, text_size, rgb(b.color), HAlign::Center);
}

// ---------------------------------------------------------------- start menu

pub struct StartMenu {
    start: Button,
    one: Button,
    two: Button,
    how: Button,
    pub selected: GameMode,
}

impl StartMenu {
    pub fn new(input: &Input) -> StartMenu {
        StartMenu {
            start: Button::new("Start Game", W / 2.0, H / 2.0 + 125.0, GREY, 350.0, 50.0, GREEN, input),
            one: Button::new("1 Player vs Robot", W / 2.0, H / 2.0 - 100.0, GREY, 420.0, 50.0, 0x00FFFF, input),
            two: Button::new("2 Players", W / 2.0, H / 2.0 - 25.0, GREY, 420.0, 50.0, 0xFF00FF, input),
            how: Button::new("How to play", W / 2.0, H - 100.0, GREY, 380.0, 50.0, 0xFFFFFF, input),
            selected: GameMode::OnePlayer,
        }
    }

    pub fn update(&mut self, input: &Input) -> Option<Transition> {
        if self.one.is_clicked(input) || input.tapped(Tap::Num1) {
            self.selected = GameMode::OnePlayer;
        }
        if self.two.is_clicked(input) || input.tapped(Tap::Num2) {
            self.selected = GameMode::TwoPlayer;
        }
        if self.start.is_clicked(input) || input.tapped(Tap::Enter) {
            return Some(Transition::StartRun(self.selected));
        }
        if self.how.is_clicked(input) || input.tapped(Tap::H) {
            return Some(Transition::ToHowTo);
        }
        if input.tapped(Tap::Escape) {
            return Some(Transition::Quit);
        }
        None
    }

    pub fn draw(&mut self, p: &mut Painter) {
        p.text("Furious Snake", W / 2.0, H / 4.0 - 100.0, 42.0, rgb(GREEN), HAlign::Center);
        self.one.bg = if self.selected == GameMode::OnePlayer { 0xFFFFFF } else { GREY };
        self.two.bg = if self.selected == GameMode::TwoPlayer { 0xFFFFFF } else { GREY };
        p.text("SELECT MODE", W / 2.0, H / 4.0, 32.0, rgb(GREEN), HAlign::Center);
        for b in [&self.start, &self.one, &self.two, &self.how] {
            draw_button(p, b, 32.0);
        }
    }
}

// --------------------------------------------------------------- how to play

pub struct HowTo {
    back: Button,
}

impl HowTo {
    pub fn new(input: &Input) -> HowTo {
        HowTo { back: Button::new("\u{2190} Back", 150.0, 75.0, GREY, 200.0, 50.0, 0xFFFFFF, input) }
    }

    pub fn update(&mut self, input: &Input) -> Option<Transition> {
        if self.back.is_clicked(input) || input.tapped(Tap::Escape) || input.tapped(Tap::Enter) {
            return Some(Transition::ToMenu);
        }
        None
    }

    pub fn draw(&self, p: &mut Painter) {
        let (cx, cy) = (W / 2.0, H / 2.0);
        let white = rgb(0xFFFFFF);
        p.text("HOW TO PLAY", cx, cy - 300.0, 32.0, rgb(GREEN), HAlign::Center);
        p.text("What to eat and what to avoid", cx, cy - 250.0, 28.0, white, HAlign::Center);
        p.solid_center(cx, cy - 100.0, 1200.0, 250.0, rgb(GREY));

        let l = HAlign::Left;
        p.text("Power-ups", cx - 300.0, cy - 200.0, 24.0, rgb(GREEN), l);
        p.sprite_center(sprites::HEART, cx - 330.0, cy - 160.0, 25.0, 25.0);
        p.text("= +1 Life", cx - 300.0, cy - 160.0, 24.0, white, l);
        p.sprite_center(sprites::STAR, cx - 330.0, cy - 110.0, 25.0, 25.0);
        p.text("= x2 Points", cx - 300.0, cy - 110.0, 24.0, white, l);

        p.text("Obstacles", cx + 100.0, cy - 200.0, 24.0, rgb(GREEN), l);
        p.sprite_center(sprites::PLANT, cx + 70.0, cy - 160.0, 21.0, 40.0);
        p.text("= -2 Life", cx + 100.0, cy - 160.0, 24.0, white, l);
        p.sprite_center(sprites::GHOST, cx + 70.0, cy - 110.0, 35.0, 35.0);
        p.text("= -1 Life, -5 Points", cx + 100.0, cy - 110.0, 24.0, white, l);
        p.sprite_center(sprites::TETRIS, cx + 70.0, cy - 60.0, 25.0, 25.0);
        p.text("= Game Over", cx + 100.0, cy - 60.0, 24.0, white, l);
        p.sprite_center(sprites::WALL, cx + 70.0, cy - 10.0, 25.0, 25.0);
        p.text("= Game Over", cx + 100.0, cy - 10.0, 24.0, white, l);

        let c = HAlign::Center;
        p.text(
            "Use the following keys on you keyboard\nto navigate your snake in the game",
            cx,
            cy + 80.0,
            28.0,
            white,
            c,
        );
        p.text("Player 1", cx - 200.0, cy + 160.0, 28.0, rgb(0x00FFFF), c);
        p.text("Player 2", cx + 200.0, cy + 160.0, 28.0, rgb(0xFF00FF), c);

        p.text("\u{2191}", cx - 200.0, cy + 220.0, 28.0, white, c);
        p.text("\u{2190} \u{2193} \u{2192}", cx - 200.0, cy + 280.0, 28.0, white, c);
        for (x, y) in [(-200.0, 220.0), (-200.0, 280.0), (-140.0, 280.0), (-260.0, 280.0)] {
            p.square_outline_center(cx + x, cy + y, 50.0, 2.0, white);
        }
        p.text("W", cx + 200.0, cy + 220.0, 28.0, white, c);
        p.text("A S D", cx + 200.0, cy + 280.0, 28.0, white, c);
        for (x, y) in [(200.0, 220.0), (200.0, 280.0), (140.0, 280.0), (260.0, 280.0)] {
            p.square_outline_center(cx + x, cy + y, 50.0, 2.0, white);
        }
        draw_button(p, &self.back, 28.0);
    }
}

// ----------------------------------------------------------------- countdown

pub struct CountDown {
    value: f64,
    level: u32,
}

impl CountDown {
    pub fn new(level: u32) -> CountDown {
        CountDown { value: 3.0, level }
    }

    pub fn update(&mut self, dt_ms: f64) -> Option<Transition> {
        if self.value > 0.0 {
            self.value -= dt_ms / 1000.0;
            if self.value <= 0.0 {
                self.value = 0.0;
                return Some(Transition::BeginRace(self.level));
            }
        }
        None
    }

    pub fn draw(&self, p: &mut Painter) {
        p.text("GET READY", W / 2.0, H / 4.0, 32.0, rgb(GREEN), HAlign::Center);
        let n = self.value.ceil() as i32;
        if n > 0 {
            p.text(&n.to_string(), W / 2.0, H / 3.0, 84.0, rgb(0xFFFFFF), HAlign::Center);
        }
    }
}

// ------------------------------------------------------------------- results

pub struct Results {
    level: u32,
    mode: GameMode,
    winner: u32,
    human_time_ms: f64,
    best: Option<(f64, bool)>,
    is_final: bool,
    next: Option<Button>,
    retry: Button,
    menu: Button,
}

impl Results {
    pub fn new(level: u32, mode: GameMode, winner: u32, human_time_ms: f64, best: Option<(f64, bool)>, input: &Input) -> Results {
        let is_final = level >= 3;
        let human_won = if mode == GameMode::OnePlayer { winner == 1 } else { winner != 0 };
        let next = (!is_final && human_won)
            .then(|| Button::new("Next Level", W / 2.0, H / 2.0 + 140.0, GREY, 300.0, 50.0, GREEN, input));
        Results {
            level,
            mode,
            winner,
            human_time_ms,
            best,
            is_final,
            next,
            retry: Button::new("Retry", W / 2.0, H / 2.0 + 210.0, GREY, 300.0, 50.0, 0xFDD03C, input),
            menu: Button::new("Menu", W / 2.0, H / 2.0 + 280.0, GREY, 300.0, 50.0, 0xFFFFFF, input),
        }
    }

    fn winner_text(&self) -> String {
        if self.mode == GameMode::OnePlayer {
            if self.winner == 1 { "YOU REACHED THE GOAL!".into() } else { "ROBOT WINS!".into() }
        } else {
            format!("PLAYER {} WINS!", self.winner)
        }
    }

    pub fn update(&mut self, input: &Input) -> Option<Transition> {
        if let Some(n) = &mut self.next {
            if n.is_clicked(input) || input.tapped(Tap::N) {
                return Some(Transition::StartLevel(self.level + 1));
            }
        }
        if self.retry.is_clicked(input) || input.tapped(Tap::R) {
            return Some(Transition::StartLevel(self.level));
        }
        if self.menu.is_clicked(input) || input.tapped(Tap::M) || input.tapped(Tap::Escape) {
            return Some(Transition::ToMenu);
        }
        None
    }

    pub fn draw(&self, p: &mut Painter) {
        let c = HAlign::Center;
        let title = if self.is_final { "FINAL RESULTS".to_string() } else { format!("LEVEL {} COMPLETE", self.level) };
        p.text(&title, W / 2.0, H / 6.0, 28.0, rgb(GREEN), c);
        p.text(&self.winner_text(), W / 2.0, H / 6.0 + 90.0, 44.0, rgb(0xFFFFFF), c);
        p.text(&format!("TIME  {}", format_time(self.human_time_ms)), W / 2.0, H / 2.0 - 20.0, 24.0, rgb(0x00FFFF), c);
        // Buttons inherit the last textSize set (a TS quirk).
        let mut button_size = 24.0;
        if let Some((best_ms, is_new)) = self.best {
            let s = if is_new {
                format!("NEW BEST!  {}", format_time(best_ms))
            } else {
                format!("BEST  {}", format_time(best_ms))
            };
            p.text(&s, W / 2.0, H / 2.0 + 20.0, 20.0, rgb(0xFDD03C), c);
            button_size = 20.0;
        }
        if let Some(n) = &self.next {
            draw_button(p, n, button_size);
        }
        draw_button(p, &self.retry, button_size);
        draw_button(p, &self.menu, button_size);
    }
}
