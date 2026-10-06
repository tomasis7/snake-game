//! Port of game.ts: the screen state machine, fixed-step simulation and debug grid.

use std::path::PathBuf;

use crate::board::{GameBoard, RaceEnd, TickCtx, DT};
use crate::draw::{rgba, Painter};
use crate::input::{Input, Tap};
use crate::progress::{GameMode, Progress};
use crate::race::RaceReason;
use crate::rng::Rng;
use crate::screens::{CountDown, HowTo, Results, StartMenu, Transition};
use crate::sound::Sound;

pub enum Screen {
    Menu(StartMenu),
    HowTo(HowTo),
    Countdown(CountDown),
    Board(Box<GameBoard>),
    Results(Results),
}

/// Which screen to open directly (screenshot mode).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScreenKind {
    Menu,
    HowTo,
    Countdown,
    Race,
    Results,
}

pub struct Game {
    pub screen: Screen,
    pub progress: Progress,
    /// Game clock in ms (TS `Date.now()` / `millis()`).
    pub clock: f64,
    pub rng: Rng,
    pub sounds: Vec<Sound>,
    pub input: Input,
    pub show_grid: bool,
    pub quit: bool,
    music_playing: bool,
    /// Bench/screenshot race: both snakes on robot AI, restart on the same level when it ends.
    robots_only: bool,
}

impl Game {
    /// The interactive game, opening on the menu with the background music looping.
    pub fn new(progress_path: Option<PathBuf>, rng: Rng) -> Game {
        Game::with_progress(Progress::new(progress_path), rng)
    }

    /// Like `new`, with a caller-built `Progress` (for example one backed by localStorage).
    pub fn with_progress(progress: Progress, rng: Rng) -> Game {
        let input = Input::default();
        Game {
            screen: Screen::Menu(StartMenu::new(&input)),
            progress,
            clock: 0.0,
            rng,
            sounds: vec![Sound::MusicLoop],
            input,
            show_grid: false,
            quit: false,
            music_playing: true,
            robots_only: false,
        }
    }

    /// Bench / screenshot setup: opens `kind` directly, silent, nothing persisted.
    pub fn for_screen(kind: ScreenKind, level: u32) -> Game {
        let mut g = Game::new(None, Rng::new(42));
        g.sounds.clear();
        g.music_playing = false;
        g.robots_only = true;
        g.screen = match kind {
            ScreenKind::Menu => Screen::Menu(StartMenu::new(&g.input)),
            ScreenKind::HowTo => Screen::HowTo(HowTo::new(&g.input)),
            ScreenKind::Countdown => Screen::Countdown(CountDown::new(level)),
            ScreenKind::Race => Screen::Board(Box::new(GameBoard::new(level, GameMode::OnePlayer, true))),
            ScreenKind::Results => {
                Screen::Results(Results::new(1, GameMode::OnePlayer, 1, 42300.0, Some((42300.0, true)), &g.input))
            }
        };
        g
    }

    fn start_level(&mut self, level: u32) {
        self.progress.current_level = level;
        self.screen = Screen::Countdown(CountDown::new(level));
    }

    fn apply(&mut self, t: Transition) {
        match t {
            Transition::ToMenu => self.screen = Screen::Menu(StartMenu::new(&self.input)),
            Transition::ToHowTo => self.screen = Screen::HowTo(HowTo::new(&self.input)),
            Transition::StartRun(mode) => {
                self.progress.start_run(mode);
                if !self.music_playing {
                    self.sounds.push(Sound::MusicLoop);
                    self.music_playing = true;
                }
                self.start_level(1);
            }
            Transition::StartLevel(n) => self.start_level(n),
            Transition::BeginRace(n) => {
                self.screen = Screen::Board(Box::new(GameBoard::new(n, self.progress.mode, self.robots_only)));
            }
            Transition::Quit => self.quit = true,
        }
    }

    fn finish_race(&mut self, level: u32, end: RaceEnd) {
        if self.robots_only {
            // Bench: restart the race on the same level.
            self.screen = Screen::Board(Box::new(GameBoard::new(level, GameMode::OnePlayer, true)));
            return;
        }
        let best = (end.winner == 1 && end.reason == RaceReason::Finish)
            .then(|| self.progress.record_best_time(level, end.time_ms));
        self.screen =
            Screen::Results(Results::new(level, self.progress.mode, end.winner, end.time_ms, best, &self.input));
    }

    /// One fixed 60 Hz simulation tick.
    pub fn update(&mut self) {
        if self.input.tapped(Tap::G) {
            self.show_grid = !self.show_grid;
        }
        let mut transition = None;
        let mut race_end = None;
        match &mut self.screen {
            Screen::Menu(s) => transition = s.update(&self.input),
            Screen::HowTo(s) => transition = s.update(&self.input),
            Screen::Countdown(s) => transition = s.update(DT),
            Screen::Results(s) => transition = s.update(&self.input),
            Screen::Board(b) => {
                if self.input.tapped(Tap::Escape) && !self.robots_only {
                    transition = Some(Transition::ToMenu);
                } else {
                    let mut tc =
                        TickCtx { clock: self.clock, input: &self.input, rng: &mut self.rng, sounds: &mut self.sounds };
                    race_end = b.update(&mut tc).map(|e| (b.level, e));
                }
            }
        }
        if self.sounds.contains(&Sound::MusicStop) {
            self.music_playing = false;
        }
        if let Some(t) = transition {
            self.apply(t);
        }
        if let Some((level, end)) = race_end {
            self.finish_race(level, end);
        }
        self.input.taps.clear();
        self.clock += DT;
    }

    /// `acc` is the fixed-step accumulator leftover in ms (for snake interpolation).
    pub fn draw(&mut self, p: &mut Painter, acc: f64) {
        match &mut self.screen {
            Screen::Menu(s) => s.draw(p),
            Screen::HowTo(s) => s.draw(p),
            Screen::Countdown(s) => s.draw(p),
            Screen::Results(s) => s.draw(p),
            Screen::Board(b) => b.draw(p, self.clock, acc, &mut self.rng),
        }
        if self.show_grid {
            // stroke(200, 0, 0, 100), 1 px lines every grid cell.
            let c = rgba(0xC80000, 100.0 / 255.0);
            for i in 0..=(1200 / 32) {
                p.solid(i as f64 * 32.0, 0.0, 1.0, 800.0, c);
            }
            for i in 0..=(800 / 32) {
                p.solid(0.0, i as f64 * 32.0, 1200.0, 1.0, c);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bench_race_runs_on_every_level() {
        for level in 1..=3 {
            let mut g = Game::for_screen(ScreenKind::Race, level);
            for _ in 0..60 * 120 {
                g.update();
            }
            assert!(matches!(g.screen, Screen::Board(_)));
        }
    }

    #[test]
    fn menu_flow() {
        let mut g = Game::new(None, Rng::new(1));
        g.input.taps.push(Tap::Num2);
        g.update();
        g.input.taps.push(Tap::Enter);
        g.update();
        assert!(matches!(g.screen, Screen::Countdown(_)));
        assert_eq!(g.progress.mode, GameMode::TwoPlayer);
        for _ in 0..200 {
            g.update();
        }
        assert!(matches!(g.screen, Screen::Board(_)));
    }

    #[test]
    fn results_shot_screen() {
        let g = Game::for_screen(ScreenKind::Results, 1);
        assert!(matches!(g.screen, Screen::Results(_)));
    }
}
